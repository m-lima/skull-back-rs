import {  Icon } from '../components/mod';
import {  useOccurrences, useSkulls, EpochDays, Occurrence } from '../store/mod';

import './chart.css';

import {
  Chart as ChartJS,
  TimeScale,
  LinearScale,
  PointElement,
  LineElement,
  Tooltip,
  Legend,
} from 'chart.js';
import 'chartjs-adapter-date-fns';
import { Line, Scatter, Chart as BaseChart } from 'react-chartjs-2';
// import { TypedChartComponent } from 'react-chartjs-2/dist/types';
import DatePicker from 'react-datepicker';
import { useMemo, useState } from 'react';

ChartJS.register(
  TimeScale,
  LinearScale,
  PointElement,
  LineElement,
  Tooltip,
  Legend
);

interface Datapoint {
  label: string,
  type: 'line' | 'scatter',
  data: {x: Date, y: number}[],
  backgroundColor: string,
}

const options = {
  responsive: true,
  scales: {
    x: {
      type: 'linear' as const,
    },
    xAxis: {
      type: 'time' as const,
    },
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

export const Chart = () => {
  const [start, setStart] = useState(EpochDays.today().subDays(7));
  const [end, setEnd] = useState(EpochDays.today());
  const [showFilters, setShowFilters] = useState(false);
  const [selectedSkulls, setSelectedSkulls] = useState<number[]>([]);
  const [query, setQuery] = useState<{length:number,step:number,by:number}>({length:24*60*60*1000,step:6*60*60*1000,by:1});

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
  const occurrences = useOccurrences(start, filter);

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
      console.log(query);
      console.log(filteredOccurrences);
      console.log(new Date(effectiveEnd));
      let cutPoint = 0;
      for (cutPoint = 0; cutPoint < filteredOccurrences.length && filteredOccurrences[cutPoint].millis.getTime() > effectiveEnd; cutPoint++) {}
      cutPoint = Math.max(0, cutPoint - 1);
      let occurrences = filteredOccurrences.slice(cutPoint);
      console.log(occurrences);

      const datapoints = new Map<number, {millis: Date, amount: number}[]>();

      // Loop while
      // - there are more windows to explore
      // - there are more occurrences to explore
      // - we have occurrences that are inside the range
      for (
        let windowStart = effectiveEnd;
        windowStart > start.getMillis() && occurrences.length > 0 && occurrences[0].millis.getTime() >= start.getMillis();
        windowStart -= query.step
      ) {
        const label = new Date(windowStart);
        const windowEnd = windowStart - query.length;

        cutPoint = 0;
        for (let i = 0; i < occurrences.length; i++) {
          const occ = occurrences[i];
          const millis = occ.millis.getTime();
          // Remove the head of the array, so next iteration can skip these
          if (millis > windowStart - query.step) {
            cutPoint = i;
            if (millis > windowStart) {
              continue;
            }
          } else if (millis < windowEnd) {
            break;
          }

          let skull = datapoints.get(occ.skull);
          if (!skull) {
            skull = [];
            datapoints.set(occ.skull, skull);
          }
          if (skull.length > 0 && skull[skull.length - 1].millis === label) {
            skull[skull.length - 1].amount += occ.amount;
          } else {
            skull.push({ millis: label, amount: occ.amount});
          }
        }

        if (cutPoint > 0) {
          occurrences = occurrences.slice(cutPoint);
        }
      }

      return Array.from(datapoints, ([k, v]) => {
        const maybeSkull = skulls.items.find(s => s.id === k);
        const skull = !!maybeSkull ? maybeSkull : { name: 'unknown', color: 13421772 };
        // return {
        //   label: skull.name,
        //   type: 'line',
        //   data: v.map(p => ({ x: p.millis, y: p.amount })),
        //   backgroundColor: `#${skull.color.toString(16).padStart(6, '0')}`,
        // };
        const o = {
          label: skull.name,
          type: 'line' as const,
          data: v.map(p => ({ x: p.millis, y: p.amount })),
          backgroundColor: `#${skull.color.toString(16).padStart(6, '0')}`,
          borderColor: `#${skull.color.toString(16).padStart(6, '0')}`,
        };
        console.log(o);
        return o;
      });
    },
    [filteredOccurrences, skulls.items, start, effectiveEnd, query],
  );

  const scatterData = useMemo(
    () => {
      const series = filteredOccurrences.reduce((acc, curr) => {
        const skull = curr.skull;
        let entry = acc.get(skull);
        if (!entry) {
          entry = [];
          acc.set(skull, entry);
        }
        entry.push({ millis: curr.millis, amount: curr.amount});
        return acc;
      }, new Map<number, { millis: Date, amount: number }[]>());

      return Array.from(series, ([k, v]) => {
        const maybeSkull = skulls.items.find(s => s.id === k);
        const skull = !!maybeSkull ? maybeSkull : { name: 'unknown', color: 13421772 };
        return {
          label: skull.name,
          type: 'scatter' as const,
          data: v.map(p => ({ x: p.millis, y: p.amount })),
          backgroundColor: `#${skull.color.toString(16).padStart(6, '0')}`,
        };
      });
    },
    [filteredOccurrences, skulls.items]
  );

  // const datasets = useMemo(
  //   () => ({
  //     labels: [],
  //     datasets: scatterData.concat(lineData),
  //     // datasets: lineData,
  //   }),
  //   [lineData, scatterData],
  // );


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
          </div>
          <div className='summary-filter-skulls'>
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
                <label htmlFor={s.name}>{s.name}</label>
              </div>
            )}
          </div>
        </>
      }
      {/*
    <BaseChart
      type='line'
      options={options}
      data={datasets}
    />
    */}
    <Line
      options={options}
      data={{datasets: lineData}}
    />
    <Scatter
      options={options}
      data={{datasets: scatterData}}
    />
  </>);
}
