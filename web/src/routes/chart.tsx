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
import { startOfDay } from 'date-fns';

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
    },
  },
  // grid: {
  //   lineWidth: (ctx) => {
  //     if (!ctx.tick || !ctx.tick.value) {
  //       return 1;
  //     }
  //
  //     const date = new Date(ctx.tixk.value);
  //
  //     if (date.getHours() === 0 && date.getMinutes() === 0 && date.getSeconds() === 0) {
  //       return 3;
  //     }
  //
  //     return 1;
  //   },
  // },
};

const alternatingDaysPlugin: Plugin = {
  id: 'alternatingDays',
  beforeDraw: (chart) => {
    const { ctx, chartArea: { top, bottom, left, right }, scales: { x } } = chart;

    ctx.save();
    ctx.beginPath();
    ctx.rect(left, top, right - left, bottom - top);
    ctx.clip();

    const minDate = new Date(x.min);
    const maxDate = new Date(x.max);

    // TODO: If too close, don't render weekends
    // TODO: If too far, don't render days
    // TODO: If really far, render months
    // TODO: All of these should be based on the width for the thresholds

    let currentDate = startOfDay(minDate);
    let isDark = Math.floor(currentDate.getTime() / 86400000) % 2 === 0;

    ctx.restore();
  },
};

const find = (occurrences: Occurrence[], needle: Date): number => {
  let start = 0;
  if (occurrences.length < 2 || occurrences[start].millis <= needle) {
    return start;
  }

  let end = occurrences.length - 1;
  if (occurrences[end].millis >= needle) {
    return end;
  }

  while (start <= end) {
    const idx = Math.floor((start + end) / 2);
    const curr = occurrences[idx].millis;

    if (curr > needle) {
      start = idx + 1;
    } else {
      end = idx - 1;
    }
  }

  return start;
};

class Timeframe {
  value: number;
  unit: number;

  constructor(value: number, unit: 'w' | 'd' | 'h' | 'm' | 's') {
    this.value = value;
    this.unit = Timeframe.unitToNumber(unit);
  }

  valueOf() {
    return this.value * this.unit;
  }

  static unitToNumber(value: 'w' | 'd' | 'h' | 'm' | 's') {
    switch (value) {
      case 'w': return 7 * 24 * 60 * 60 * 1000;
      case 'd': return 24 * 60 * 60 * 1000;
      case 'h': return 60 * 60 * 1000;
      case 'm': return 60 * 1000;
      case 's': return 1000;
    }
  }

  static numberToUnit(value: number) {
    switch (value) {
      case 7 * 24 * 60 * 60 * 1000: return 'w';
      case 24 * 60 * 60 * 1000: return 'd';
      case 60 * 60 * 1000: return 'h';
      case 60 * 1000: return 'm';
      case 1000: return 's';
    }
  }
}

const day = 24 * 60 * 60 * 1000;

export const Chart = () => {
  const [start, setStart] = useState(EpochDays.today().subDays(7));
  const [end, setEnd] = useState(EpochDays.today());
  const [showLimits, setShowLimits] = useState(false);
  const [selectedSkulls, setSelectedSkulls] = useState<number[]>([]);
  const [queryLength, setQueryLength] = useState(1 * day);
  const [queryStep, setQueryStep] = useState(1 * 0.25 * day);
  const [queryByOverride, setQueryByOverride] = useState(false);
  const [queryBy, setQueryBy] = useState(1 * day);

  // const parsedQuery = useMemo(
  //   () => {
  //     const parts = query.split('/');
  //     if (parts.length > 3) {
  //       return undefined;
  //     }
  //
  //
  //   },
  //   [query]
  // );

  const effectiveStart = useMemo(() => start.getMillis() - queryLength, [start, queryLength]);
  const effectiveEnd = useMemo(() => end.addDays(1).getMillis(), [end]);

  // TODO: Need to add date ahead to accomodate the mid
  // Basically, the start and end dates should reference the midpoint of each window
  const filter = useMemo(
    () => {
      return (o: Occurrence) => o.millis.getTime() <= effectiveEnd;
    },
    [effectiveEnd],
  );

  const skulls = useSkulls();
  const occurrences = useOccurrences(effectiveStart, filter);

  const filteredOccurrences = useMemo(
    () => {
      const filtered = occurrences.items.filter(o => selectedSkulls.indexOf(o.skull) < 0);
      filtered.sort((a, b) => {
        if (a.millis > b.millis) {
          return -1;
        } else if (a.millis < b.millis) {
          return 1;
        } else {
          if (a.skull > b.skull) {
            return -1;
          } else if (a.skull < b.skull) {
            return 1;
          } else {
            return 0;
          }
        }
      });
      return filtered;
    },
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
        {/*
          <div className='summary-filter-input'>
            <b>Length</b>
            <input
              id={Number(queryLength) ? '' : 'invalid'}
              type='text'
              inputMode='decimal'
              min={0}
              step={0.1}
              value={queryLength}
              onChange={e => setQueryLength(Number(e.target.value))}
            />
            <select
              value={stagedValue.skull.name}
              disabled={markedForDeletion}
              onChange={e => stageSkull(e.target.value)}
            >
              {props.skulls.map((s, i) => (
                <option key={i} value={s.name}>
                  {s.name}
                </option>
              ))}
            </select>
          </div>
            */}
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
      <Line
        style={{ minHeight: 0, minWidth: 0 }}
        options={options}
        data={{
          labels: [],
          datasets: lineData,
        }}
      />
    </>);
}
