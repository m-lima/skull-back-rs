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
import { Scatter } from 'react-chartjs-2';
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

const options = {
  responsive: true,
  scales: {
    xAxis: {
      type: 'time' as const,
    },
  },
};

export const Chart = () => {
  const [start, setStart] = useState(EpochDays.today().subDays(7));
  const [end, setEnd] = useState(EpochDays.today());
  const [showFilters, setShowFilters] = useState(false);
  const [selectedSkulls, setSelectedSkulls] = useState<number[]>([]);

  const filter = useMemo(
    () => {
      const effectiveEnd = end.addDays(1).getMillis();
      return (o: Occurrence) => o.millis.getTime() <= effectiveEnd;
    },
    [end],
  );

  const skulls = useSkulls();
  const occurrences = useOccurrences(start, filter);

  const filteredOccurrences = useMemo(
    () => occurrences.items.filter(o => selectedSkulls.indexOf(o.skull) < 0),
    [occurrences.items, selectedSkulls],
  );

  const data = useMemo(
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

      const datasets = Array.from(series, ([k, v]) => {
        const maybeSkull = skulls.items.find(s => s.id === k);
        const skull = !!maybeSkull ? maybeSkull : { name: 'unknown', color: 13421772 };
        return {
          label: skull.name,
          data: v.map(p => ({ x: p.millis, y: p.amount })),
          backgroundColor: `#${skull.color.toString(16).padStart(6, '0')}`,
        };
      });
      for (const serie in series.entries) {
        console.log(serie)
      }
      return {
        labels: [],
        datasets,
      };
    },
    [filteredOccurrences, skulls.items]
  );

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
    <Scatter
    options={options}
    data={data}
  />
  </>);
}
