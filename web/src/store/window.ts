const VALID_UNITS = ['m', 'w', 'd', 'h'] as const;
type Unit = (typeof VALID_UNITS)[number];

class Timeframe {
  amount: number;
  unit: Unit;

  constructor(amount: number, unit: Unit) {
    this.amount = amount;
    this.unit = unit;
  }

  readonly toString = () => `${this.amount}${this.unit}`;
  readonly valueOf = () => this.amount * Timeframe.unitToNumber(this.unit);

  static fromString(timeframe: string) {
    let trimmed = timeframe.trim();
    if (trimmed.length < 2) {
      console.log(`${timeframe} has bad length: ${trimmed.length}`);
      return undefined;
    }

    const unit = trimmed[trimmed.length - 1];
    trimmed = trimmed.slice(0, -1);
    if (!Timeframe.isUnit(unit)) {
      console.log(`${timeframe} has bad unit: ${unit}`);
      return undefined;
    }

    const amount = Number(trimmed);
    if (!amount) {
      console.log(`${timeframe} has bad amount: ${trimmed}`);
      return undefined;
    }

    return new Timeframe(amount, unit);
  }

  static isUnit = (value: string): value is Unit =>
    (VALID_UNITS as readonly string[]).includes(value);

  static unitToNumber(unit: Unit) {
    switch (unit) {
      case 'm':
        return 31 * 24 * 60 * 60 * 1000;
      case 'w':
        return 7 * 24 * 60 * 60 * 1000;
      case 'd':
        return 24 * 60 * 60 * 1000;
      case 'h':
        return 60 * 60 * 1000;
    }
  }
}

interface IntoTimeframe {
  amount: number;
  unit: Unit;
}

export class Window {
  length: Timeframe;
  step: Timeframe;
  by: Timeframe;

  constructor(length: IntoTimeframe, step: IntoTimeframe, by: IntoTimeframe) {
    this.length = new Timeframe(length.amount, length.unit);
    this.step = new Timeframe(step.amount, step.unit);
    this.by = new Timeframe(by.amount, by.unit);
  }

  readonly toString = () =>
    `${this.length.toString()}/${this.step.toString()}/${this.by.toString()}`;

  readonly getLength = () => this.length.valueOf();
  readonly getStep = () => this.step.valueOf();
  readonly getBy = () => this.by.valueOf();

  static fromString(value: string) {
    const parts = value.split('/');
    if (parts.length !== 3) {
      console.log(`'${value}' is no three parts`);
      return undefined;
    }

    const length = Timeframe.fromString(parts[0]);
    if (length === undefined) {
      console.log(`'${value}' has bad length: ${parts[0]}`);
      return undefined;
    }

    const step = Timeframe.fromString(parts[1]);
    if (step === undefined || step > length) {
      console.log(`'${value}' has bad step: ${parts[1]}`);
      return undefined;
    }

    const by = Timeframe.fromString(parts[2]);
    if (by === undefined || by > length) {
      console.log(`'${value}' has bad by: ${parts[2]}`);
      return undefined;
    }

    return new Window(length, step, by);
  }
}
