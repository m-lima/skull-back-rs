import * as datefns from 'date-fns';

export interface Skull {
  id: number,
  name: string,
  color: number,
  icon: string,
  price: number,
  limit?: number,
}


export interface RawQuick {
  id: number,
  skull: number,
  amount: number,
}

export interface Quick {
  id: number,
  skull: Skull,
  amount: number,
}

export interface Occurrence {
  id: number,
  skull: number,
  amount: number,
  millis: Date,
}

export interface ProtoOccurrence {
  skull: number,
  amount: number,
  millis: Date,
}

export interface Response<T> extends StoreStatus {
  items: T[],
}

export interface StoreStatus {
  pending: boolean,
  error?: any,
}

export class EpochDays {
  private millis: number;

  public constructor(date: EpochDays | Date | number) {
    if (date instanceof EpochDays) {
      this.millis = date.millis;
    } else {
      this.millis = datefns.setHours(EpochDays.clampToHours(date), 5).getTime();
    }
  }

  private static clampToHours(date: Date | number) {
    return datefns.setMinutes(datefns.setSeconds(datefns.setMilliseconds(date, 0), 0), 0);
  }

  public static today(): EpochDays {
    return new EpochDays(new Date());
  }

  public static toBoundary(date: EpochDays | Date | number) {
    const millis = (date instanceof EpochDays)
      ? date.millis
      : EpochDays.clampToHours(date);

    return datefns.subHours(millis, 5).getTime();
  }

  public addDays(days: number): EpochDays {
    const out = new EpochDays(this);
    out.millis = datefns.addDays(out.millis, days).getTime();
    return out;
  }

  public subDays(days: number): EpochDays {
    const out = new EpochDays(this);
    out.millis = datefns.subDays(out.millis, days).getTime();
    return out;
  }

  public valueOf(): number {
    return this.millis;
  }

  public getMillis(): number {
    return this.millis;
  }
}

export namespace sealed {
  export const makeSkull = (s: [number, string, number, string, number, number?]) => {
    return {
      id: s[0],
      name: s[1],
      color: s[2],
      icon: s[3],
      price: s[4],
      limit: s[5],
    };
  };

  export const makeRawQuick = (q: [number, number, number]) => {
    return {
      id: q[0],
      skull: q[1],
      amount: Number(q[2].toFixed(3)),
    }
  };

  export const makeOccurrence = (o: [number, number, number, number]) => {
    return {
      id: o[0],
      skull: o[1],
      amount: Number(o[2].toFixed(3)),
      millis: new Date(o[3]),
    }
  };
}
