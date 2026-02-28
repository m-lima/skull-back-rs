import { Socket, SocketState } from '../socket';
import { sealed as querySealed } from './query';
import { sealed as modelSealed, Occurrence, Quick, RawQuick, Skull, EpochDays, ProtoOccurrence } from './model';

export class Store {
  private readonly socket: Socket;

  private skulls: Skull[];
  private quicks: Quick[];
  private occurrences: Occurrence[];

  private hasSkulls: boolean;
  private hasQuicks: boolean;
  private hasOccurrencesSince?: EpochDays;

  private fetchingSkulls: boolean;
  private fetchingQuicks: boolean;
  private fetchingOccurrences: boolean;

  private pendingRequests: (() => void)[] = [];

  private readonly skullListeners: Listener<Skull[]>[];
  private readonly quickListeners: Listener<Quick[]>[];
  private readonly occurrenceListeners: Listener<Occurrence[]>[];

  public constructor(socket: Socket) {
    this.socket = socket;
    this.socket.registerHandler(this.changeHandler);
    this.socket.registerStateListener(state => {
      if (state === SocketState.Open) {
        this.ensureAll();

        for (const request of this.pendingRequests) {
          request();
        }
        this.pendingRequests = [];
      } else if (state !== SocketState.Connecting) {
        this.pendingRequests = [];
      }
    });

    this.skulls = [];
    this.quicks = [];
    this.occurrences = [];

    this.hasSkulls = false;
    this.hasQuicks = false;
    this.hasOccurrencesSince = undefined;

    this.fetchingSkulls = false;
    this.fetchingQuicks = false;
    this.fetchingOccurrences = false;

    this.skullListeners = [];
    this.quickListeners = [];
    this.occurrenceListeners = [];
  }

  public getSocket() {
    return this.socket;
  }

  public registerSkullListener(handler: (skulls: Skull[]) => void) {
    const listener = new Listener(handler);
    this.skullListeners.push(listener);
    return listener;
  }

  public removeSkullListener(listener: Listener<Skull[]>) {
    const index = this.skullListeners.indexOf(listener);
    if (index >= 0) {
      this.skullListeners.splice(index, 1);
    }
  }

  public registerQuickListener(handler: (quicks: Quick[]) => void) {
    const listener = new Listener(handler);
    this.quickListeners.push(listener);
    return listener;
  }

  public removeQuickListener(listener: Listener<Quick[]>) {
    const index = this.quickListeners.indexOf(listener);
    if (index >= 0) {
      this.quickListeners.splice(index, 1);
    }
  }

  public registerOccurrenceListener(handler: (occurrences: Occurrence[]) => void) {
    const listener = new Listener(handler);
    this.occurrenceListeners.push(listener);
    return listener;
  }

  public removeOccurrenceListener(listener: Listener<Occurrence[]>) {
    const index = this.occurrenceListeners.indexOf(listener);
    if (index >= 0) {
      this.occurrenceListeners.splice(index, 1);
    }
  }

  public isSkullsLoaded() {
    return this.hasSkulls;
  }

  public async ensureSkulls() {
    if (this.hasSkulls || this.fetchingSkulls) {
      return;
    }

    this.fetchingSkulls = true;

    return this.wrapRequest(() =>
      querySealed.getSkulls(this.socket)
        .then(skulls => {
          this.hasSkulls = true;
          this.setSkulls(skulls, true);
        })
        .finally(() => {
          this.fetchingSkulls = false;
        })
    );
  }

  public getSkulls() {
    return this.skulls;
  }

  public isQuicksLoaded() {
    return this.hasQuicks;
  }

  public async ensureQuicks() {
    if (this.hasQuicks || this.fetchingQuicks) {
      return;
    }

    this.fetchingQuicks = true;

    return this.wrapRequest(() =>
      querySealed.getQuicks(this.socket)
        .then(quicks => {
          this.hasQuicks = true;
          this.setQuicks(quicks, true);
        })
        .finally(() => {
          this.fetchingQuicks = false;
        })
    );
  }

  public getQuicks() {
    return this.quicks;
  }

  public isOccurrencesLoadedSince(start: EpochDays) {
    return !!this.hasOccurrencesSince && this.hasOccurrencesSince <= start;
  }

  public async ensureOccurrences(start: EpochDays) {
    if (this.isOccurrencesLoadedSince(start) || this.fetchingOccurrences) {
      return;
    }

    this.fetchingOccurrences = true;

    return this.wrapRequest(() =>
      querySealed.getOccurrences(this.socket, start, this.hasOccurrencesSince)
        .then(occurrences => {
          this.hasOccurrencesSince = start;
          this.setOccurrences(occurrences, true);
        })
        .finally(() => {
          this.fetchingOccurrences = false;
        })
    );
  }

  public getOccurrences() {
    return this.occurrences;
  }

  public edit = {
    create: (occurrence: ProtoOccurrence) => this.wrapRequest(() =>
      querySealed.edit.create(this.socket, occurrence)
    ),
    update: (occurrence: Occurrence) => this.wrapRequest(() =>
      querySealed.edit.update(this.socket, occurrence)
    ),
    remove: (occurrence: Occurrence) => this.wrapRequest(() =>
      querySealed.edit.remove(this.socket, occurrence)
    ),
  };

  private setSkulls(skulls: Skull[], forceBroadcast: boolean = false) {
    if (skulls.length === 0) {
      if (forceBroadcast) {
        this.broadcastSkulls();
      }
      return;
    }

    let quicksUpdate = false;
    for (const skull of skulls) {
      const index = this.skulls.findIndex(s => s.id === skull.id);
      if (index < 0) {
        this.skulls.push(skull);
      } else {
        this.skulls[index] = skull;
      }

      quicksUpdate = this.updateQuicks(skull) || quicksUpdate;
    }

    this.broadcastSkulls();
    if (quicksUpdate) {
      this.broadcastQuicks();
    }
  }

  private removeSkull(id: number) {
    const index = this.skulls.findIndex(s => s.id === id);
    if (index >= 0) {
      this.skulls.splice(index, 1);

      let quicksLength = this.quicks.length;
      this.quicks = this.quicks.filter(q => q.skull.id !== id);
      if (quicksLength > this.quicks.length) {
        this.broadcastQuicks();
      }

      this.broadcastSkulls();
    }
  }

  private setQuicks(rawQuicks: RawQuick[], forceBroadcast: boolean = false) {
    if (rawQuicks.length === 0) {
      if (forceBroadcast) {
        this.broadcastQuicks();
      }
      return;
    }

    for (const rawQuick of rawQuicks) {
      const skull = this.skulls.find(s => s.id === rawQuick.skull);
      if (!skull) {
        return;
      }

      const quick = { ...rawQuick, skull };
      const index = this.quicks.findIndex(q => q.skull === quick.skull && q.amount === quick.amount);
      if (index < 0) {
        this.quicks.push(quick);
      } else {
        this.quicks[index] = quick;
      }
    }

    this.broadcastQuicks();
  }

  private setOccurrences(occurrences: Occurrence[], forceBroadcast: boolean = false) {
    if (occurrences.length === 0) {
      if (forceBroadcast) {
        this.broadcastOccurrences();
      }
      return;
    }

    for (const occurrence of occurrences) {
      const index = this.occurrences.findIndex(o => o.id === occurrence.id);
      if (index < 0) {
        this.occurrences.push(occurrence);
      } else {
        this.occurrences[index] = occurrence;
      }
    }

    this.broadcastOccurrences();
  }

  private removeOccurrence(id: number) {
    const index = this.occurrences.findIndex(q => q.id === id);
    if (index >= 0) {
      this.occurrences.splice(index, 1);
      this.broadcastOccurrences();
    }
  }

  private broadcastSkulls() {
    for (const listener of this.skullListeners) {
      listener.handler([...this.skulls]);
    }
  }

  private broadcastQuicks() {
    for (const listener of this.quickListeners) {
      listener.handler(this.quicks);
    }
  }

  private broadcastOccurrences() {
    for (const listener of this.occurrenceListeners) {
      listener.handler(this.occurrences);
    }
  }

  private updateQuicks(skull: Skull) {
    let modified = false;

    for (const quick of this.quicks) {
      if (quick.skull.id === skull.id) {
        modified = true;
        quick.skull = skull;
      }
    }

    return modified;
  }

  private wrapRequest<T>(
    request: () => Promise<T>
  ): Promise<T> {
    if (this.socket.getState() === SocketState.Open) {
      return request();
    }

    return new Promise((accept, reject) => {
      this.pendingRequests.push(() => request().then(ok => accept(ok)).catch(err => reject(err)));
    });
  }

  private ensureAll() {
    if (this.hasSkulls) {
      querySealed.getSkulls(this.socket)
        .then(skulls => this.setSkulls(skulls));
    }
    if (this.hasQuicks) {
      querySealed.getQuicks(this.socket)
        .then(quicks => this.setQuicks(quicks));
    }
    if (!!this.hasOccurrencesSince) {
      querySealed.getOccurrences(this.socket, this.hasOccurrencesSince)
        .then(occurrences => this.setOccurrences(occurrences));
    }
  }

  private readonly changeHandler = (message: any) => {
    if ('push' in message) {
      const push = message.push;
      if ('skullCreated' in push) {
        this.setSkulls([modelSealed.makeSkull(push.skullCreated)]);
        return true;
      } else if ('skullUpdated' in push) {
        this.setSkulls([modelSealed.makeSkull(push.skullUpdated)]);
        return true;
      } else if ('skullDeleted' in push) {
        this.removeSkull(push.skullDeleted);
        return true;
      } else if ('occurrencesCreated' in push) {
        this.setOccurrences(push.occurrencesCreated.map(modelSealed.makeOccurrence));
        return true;
      } else if ('occurrenceUpdated' in push) {
        this.setOccurrences([modelSealed.makeOccurrence(push.occurrenceUpdated)]);
        return true;
      } else if ('occurrenceDeleted' in push) {
        this.removeOccurrence(push.occurrenceDeleted);
        return true;
      }
    }
    return false;
  };
}

class Listener<T> {
  readonly handler: (data: T) => void;

  constructor(handler: (data: T) => void) {
    this.handler = handler;
  }
}
