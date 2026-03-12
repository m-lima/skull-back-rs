import { Icon } from '../components/mod';
import { skullColor, useOccurrences, useSkulls, EpochDays, Occurrence } from '../store/mod';

import './chart.css';

import {
  Chart as ChartJS,
  LineElement,
  LinearScale,
  Plugin,
  PointElement,
  TimeScale,
  Tooltip,
} from 'chart.js';
import 'chartjs-adapter-date-fns';
import { Line } from 'react-chartjs-2';
import DatePicker from 'react-datepicker';
import { useMemo, useState } from 'react';
import * as datefns from 'date-fns';

ChartJS.register(
  LineElement,
  LinearScale,
  PointElement,
  TimeScale,
  Tooltip,
);

const options = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    x: {
      type: 'time' as const,
      time: {
        tooltipFormat: "E, d MMM yy, k'h'",
      },
    },
  },
};

const alternatingDaysPlugin: Plugin = {
  id: 'alternatingDays',
  beforeDraw: (chart) => {
    const { ctx, canvas, chartArea: { top, bottom, left, right }, scales: { x } } = chart;

    ctx.save();
    ctx.beginPath();
    ctx.rect(left, top, right - left, bottom - top);
    ctx.clip();

    const styles = getComputedStyle(canvas);
    const dark = styles.getPropertyValue('--shade-1').trim();
    const bright = styles.getPropertyValue('--shade-2').trim();

    const span = datefns.differenceInDays(x.max, x.min);
    const [stepper, firstStep]: [(date: Date) => Date, (date: Date) => Date] =
      span >= 730
        ? [a => datefns.subQuarters(a, 1), datefns.startOfQuarter]
        : span >= 90
          ? [a => datefns.subMonths(a, 1), datefns.startOfMonth]
          : span >= 30
            ? [a => datefns.subWeeks(a, 1), datefns.startOfWeek]
            : [a => datefns.subDays(a, 1), datefns.startOfDay]
    const minDay = new Date(x.min);

    let currDay = new Date(x.max);
    let prevDay = firstStep(currDay);
    let isDark = true;
    while (currDay > minDay) {
      const startX = x.getPixelForValue(prevDay.getTime());
      const endX = x.getPixelForValue(currDay.getTime());

      ctx.fillStyle = isDark ? dark : bright;
      ctx.fillRect(startX, top, endX - startX, bottom - top)

      currDay = prevDay;
      prevDay = stepper(prevDay);
      isDark = !isDark;
    }

    ctx.restore();
  },
};

const VALID_UNITS = ['m', 'w', 'd', 'h'] as const;
type Unit = typeof VALID_UNITS[number];

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

  static isUnit = (value: string): value is Unit => (VALID_UNITS as readonly string[]).includes(value);

  static unitToNumber(unit: Unit) {
    switch (unit) {
      case 'm': return 31 * 24 * 60 * 60 * 1000;
      case 'w': return 7 * 24 * 60 * 60 * 1000;
      case 'd': return 24 * 60 * 60 * 1000;
      case 'h': return 60 * 60 * 1000;
    }
  }
}

class QueryWindow {
  length: Timeframe;
  step: Timeframe;
  by: Timeframe;

  constructor(length: Timeframe, step: Timeframe, by: Timeframe) {
    this.length = length;
    this.step = step;
    this.by = by;
  }

  readonly toString = () => `${this.length.toString()}/${this.step.toString()}/${this.by.toString()}`;

  readonly getLength = () => this.length.valueOf();
  readonly getStep = () => this.step.valueOf();
  readonly getBy = () => this.by.valueOf();

  static fromString(value: string) {
    const parts = value.split('/');
    if (parts.length !== 3) {
      console.log(`'${value}' is no three parts`)
      return undefined;
    }

    const length = Timeframe.fromString(parts[0]);
    if (length === undefined) {
      console.log(`'${value}' has bad length: ${parts[0]}`)
      return undefined;
    }

    const step = Timeframe.fromString(parts[1]);
    if (step === undefined || step > length) {
      console.log(`'${value}' has bad step: ${parts[1]}`)
      return undefined;
    }

    const by = Timeframe.fromString(parts[2]);
    if (by === undefined || by > length) {
      console.log(`'${value}' has bad by: ${parts[2]}`)
      return undefined;
    }

    return new QueryWindow(length, step, by);
  }
}

export const Chart = () => {
  const [start, setStart] = useState(new EpochDays(datefns.subMonths(new Date(), 6)));
  const [end, setEnd] = useState(EpochDays.today());
  const [showFilters, setShowFilters] = useState(false);
  const [showLimits, setShowLimits] = useState(false);
  const [selectedSkulls, setSelectedSkulls] = useState<number[]>([]);

  const [query, setQuery] = useState(() => new QueryWindow(new Timeframe(15, 'd'), new Timeframe(1, 'd'), new Timeframe(1, 'd')));
  const [queryStr, setQueryStr] = useState(query.toString());

  const [queryLength, queryStep, queryBy] = useMemo(() => [query.getLength(), query.getStep(), query.getBy()], [query]);

  const effectiveStart = useMemo(() => start.getMillis() - queryLength, [start, queryLength]);
  const effectiveEnd = useMemo(() => end.addDays(1).getMillis(), [end]);

  const filter = useMemo(() => (o: Occurrence) => o.millis.getTime() <= effectiveEnd, [effectiveEnd]);

  const skulls = useSkulls();
  const occurrences = useOccurrences(effectiveStart, filter);

  const filteredOccurrences = useMemo(
    () => occurrences.items.filter(o => selectedSkulls.indexOf(o.skull) < 0),
    [occurrences.items, selectedSkulls],
  );

  const lineData = useMemo(
    () => {
      let cutPoint = 0;
      for (cutPoint = 0; cutPoint < filteredOccurrences.length && filteredOccurrences[cutPoint].millis.getTime() > effectiveEnd; cutPoint++) { }
      cutPoint = Math.max(0, cutPoint - 1);
      let occurrences = filteredOccurrences.slice(cutPoint);

      const datapoints = new Map<number, { millis: Date, amount: number }[]>();
      skulls
        .items
        .filter(s => selectedSkulls.find(id => id === s.id) === undefined)
        .forEach(s => datapoints.set(s.id, []));

      // Loop while
      // - there are more windows to explore
      // - there are more occurrences to explore
      // - we have occurrences that are inside the range
      for (
        let windowTop = effectiveEnd;
        windowTop >= start.getMillis() && occurrences.length > 0 && occurrences[0].millis.getTime() >= effectiveStart;
        windowTop -= queryStep
      ) {
        const label = new Date(windowTop);
        const windowBottom = windowTop - queryLength;

        datapoints.forEach((amounts, _) => amounts.push({ millis: label, amount: 0 }));

        cutPoint = 0;
        for (let i = 0; i < occurrences.length; i++) {
          const occ = occurrences[i];
          const millis = occ.millis.getTime();
          // Remove the head of the array, so next iteration can skip these
          if (millis > windowTop - queryStep) {
            cutPoint = i;
            if (millis > windowTop) {
              continue;
            }
          } else if (millis < windowBottom) {
            break;
          }

          let skull = datapoints.get(occ.skull);
          if (!skull) {
            skull = [];
            datapoints.set(occ.skull, skull);
          }
          if (skull.length > 0 && skull[skull.length - 1].millis === label) {
            skull[skull.length - 1].amount += occ.amount * (queryBy / queryLength);
          } else {
            skull.push({ millis: label, amount: occ.amount });
          }
        }

        if (cutPoint > 0) {
          occurrences = occurrences.slice(cutPoint);
        }
      }

      const data = Array.from(datapoints, ([k, v]) => {
        const skull = skulls.items.find(s => s.id === k);
        if (skull === undefined || v.length === 0) {
          return [];
        }
        return [
          {
            type: 'line' as const,
            data: v.map(p => ({ x: p.millis, y: p.amount })),
            pointRadius: 0,
            pointHoverRadius: 5,
            pointHitRadius: 20,
            backgroundColor: skullColor(skull),
            borderColor: skullColor(skull),
          },
          {
            hidden: !showLimits || skull.limit === undefined,
            type: 'line' as const,
            data: [
              { x: new Date(start.getMillis()), y: skull.limit },
              { x: new Date(effectiveEnd), y: skull.limit },
            ],
            pointRadius: 0,
            borderColor: skullColor(skull, 0.6),
            borderDash: [5, 10],
          },
        ];
      }).flat();

      return data;
    },
    [filteredOccurrences, selectedSkulls, showLimits, skulls.items, start, effectiveStart, effectiveEnd, queryLength, queryStep, queryBy],
  );

  // TODO: Move filter to separate component
  // TODO: Search for "summary" for further cleanup
  return (
    <>
      <div
        className='summary-filter-toggle'
        onClick={() => setShowFilters(!showFilters)}
      >
        <span id='label'>Filter</span>
        <Icon icon={showFilters ? 'fas fa-caret-up' : 'fas fa-caret-down'} />
      </div>
      {showFilters &&
        <>
          <div className='summary-filter-inputs'>
            <div className='summary-filter-input'>
              <b>Start</b>
              <DatePicker
                selected={new Date(start.getMillis())}
                dateFormat='dd/MM/yyyy'
                popperPlacement='bottom'
                onChange={d => d && setStart(new EpochDays(d))}
              />
            </div>
            <div className='summary-filter-input'>
              <b>End</b>
              <DatePicker
                selected={new Date(end.getMillis())}
                dateFormat='dd/MM/yyyy'
                popperPlacement='bottom'
                onChange={d => d && setEnd(new EpochDays(d))}
              />
            </div>
            <div className='summary-filter-input'>
              <b>Window</b>
              <input
                type='text'
                value={queryStr}
                onChange={e => {
                  setQueryStr(e.target.value)
                  const parsed = QueryWindow.fromString(e.target.value);
                  if (parsed !== undefined) {
                    setQuery(parsed);
                  }
                }}
                onBlur={() => setQueryStr(query.toString())}
              />
            </div>
            {/*
            <div className='summary-filter-input-small'>
              <div className='summary-filter-input'>
                <b>Window</b>
                <input
                  // id={Number(queryLengthStr) ? '' : 'invalid'}
                  type='text'
                  inputMode='numeric'
                  min={0}
                  step={1}
                  value={queryLengthStr}
                  onChange={e => {
                    setQueryLengthStr(e.target.value)
                    const asNumber = Math.floor(Number(e.target.value));
                    asNumber && setQueryLength(asNumber * day);
                  }}
                  onBlur={() => setQueryLengthStr(Math.floor(queryLength / day).toString())}
                />
              </div>
              <div className='summary-filter-input'>
                <b>Step</b>
                <input
                  id={Number(queryLengthStr) ? '' : 'invalid'}
                  type='text'
                  inputMode='numeric'
                  min={0}
                  step={1}
                  value={queryLengthStr}
                  onChange={e => {
                    setQueryLengthStr(e.target.value)
                    const asNumber = Math.floor(Number(e.target.value));
                    asNumber && setQueryLength(asNumber * day);
                  }}
                  onBlur={() => setQueryLengthStr(Math.floor(queryLength / day).toString())}
                />
              </div>
              <div className='summary-filter-input'>
                <b>By</b>
                <input
                  id={Number(queryLengthStr) ? '' : 'invalid'}
                  type='text'
                  inputMode='numeric'
                  min={0}
                  step={1}
                  value={queryLengthStr}
                  onChange={e => {
                    setQueryLengthStr(e.target.value)
                    const asNumber = Math.floor(Number(e.target.value));
                    asNumber && setQueryLength(asNumber * day);
                  }}
                  onBlur={() => setQueryLengthStr(Math.floor(queryLength / day).toString())}
                />
              </div>
            </div>*/}
          </div>
          <div className='summary-filter-skulls'>
            <div>
              <input
                type='checkbox'
                checked={showLimits}
                onChange={() => setShowLimits(!showLimits)}
              />
              <label>Limits</label>
            </div>
            {skulls.items.map((s, i) =>
              <div key={i}>
                <input
                  id={s.name}
                  type='checkbox'
                  defaultChecked={selectedSkulls.find(id => id === s.id) === undefined}
                  onChange={() => {
                    const index = selectedSkulls.findIndex(id => id === s.id);
                    if (index < 0) {
                      selectedSkulls.push(s.id);
                    } else {
                      selectedSkulls.splice(index, 1);
                    }
                    setSelectedSkulls([...selectedSkulls]);
                  }}
                />
                <label htmlFor={s.name} style={{ color: skullColor(s) }}>{s.name}</label>
              </div>
            )}
          </div>
        </>
      }
      <div style={{ flex: 1, minHeight: 0, minWidth: 0 }} >
        <Line
          options={options}
          data={{
            labels: [],
            datasets: lineData,
          }}
          plugins={[alternatingDaysPlugin]}
        />
      </div>
    </>);
}
