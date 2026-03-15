import * as datefns from 'date-fns';

export interface Skull {
  id: number;
  name: string;
  color: number;
  icon: string;
  price: number;
  limit?: number;
}

const opacityToHex = (opacity?: string | number) =>
  opacity === undefined
    ? ''
    : typeof opacity === 'string'
      ? opacity
      : Math.floor(255 * opacity)
          .toString(16)
          .padStart(2, '0');

export const skullColor = (skull: { color: number }, opacity?: number) =>
  `#${skull.color.toString(16).padStart(6, '0')}${opacityToHex(opacity)}`;

export interface RawQuick {
  skull: number;
  amount: number;
}

export interface Quick {
  skull: Skull;
  amount: number;
}

export interface Occurrence {
  id: number;
  skull: number;
  amount: number;
  millis: Date;
}

export interface ProtoOccurrence {
  skull: number;
  amount: number;
  millis: Date;
}

export interface Response<T> extends StoreStatus {
  items: T[];
}

export interface StoreStatus {
  pending: boolean;
  error?: unknown;
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
    const millis = date instanceof EpochDays ? date.millis : EpochDays.clampToHours(date);

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

export const sealed = {
  isSkullTuple: (v: unknown): v is [number, string, number, string, number, number?] =>
    v instanceof Array &&
    (v.length === 5 || v.length === 6) &&
    typeof v[0] === 'number' &&
    typeof v[1] === 'string' &&
    typeof v[2] === 'number' &&
    typeof v[3] === 'string' &&
    typeof v[4] === 'number' &&
    (v.length === 5 || typeof v[5] === 'number'),
  makeSkull: (s: [number, string, number, string, number, number?]) => {
    return {
      id: s[0],
      name: s[1],
      color: s[2],
      icon: s[3],
      price: s[4],
      limit: s[5],
    };
  },

  isQuickTuple: (v: unknown): v is [number, number] =>
    v instanceof Array && v.length === 2 && typeof v[0] === 'number' && typeof v[1] === 'number',
  makeRawQuick: (q: [number, number]) => {
    return {
      skull: q[0],
      amount: Number(q[1].toFixed(3)),
    };
  },

  isOccurrenceTuple: (v: unknown): v is [number, number, number, number] =>
    v instanceof Array &&
    v.length === 4 &&
    typeof v[0] === 'number' &&
    typeof v[1] === 'number' &&
    typeof v[2] === 'number' &&
    typeof v[3] === 'number',
  makeOccurrence: (o: [number, number, number, number]) => {
    return {
      id: o[0],
      skull: o[1],
      amount: Number(o[2].toFixed(3)),
      millis: new Date(o[3]),
    };
  },
} as const;
