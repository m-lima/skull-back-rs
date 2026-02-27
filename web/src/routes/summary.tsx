import { Banner, Edit, Icon } from '../components/mod';
import { check, useEditOccurrence, useOccurrences, useSkulls, EpochDays, Occurrence, Skull } from '../store/mod';

import './summary.css';

import * as datefns from 'date-fns';
import DatePicker from 'react-datepicker';
import { useMemo, useState } from 'react';

const renderRows = (occurrences: Occurrence[], skullMap: Map<number, Skull>, setSelected: (o: Occurrence) => void) => {
  let id = true;
  let day = Number.NaN;

  const formatDate = (date: Date) => datefns.format(date, 'dd/MM/yy HH:mm');

  const renderRow = (index: number, occurrence: Occurrence) => {
    const skull = skullMap.get(occurrence.skull);
    if (!skull) {
      return <></>;
    }

    const currDay = EpochDays.toBoundary(occurrence.millis);
    if (!datefns.isSameDay(currDay, day)) {
      day = currDay;
      id = !id;
    }

    return (
      <tr
        id={id ? 'bright' : 'dark'}
        key={index}
        onClick={() => setSelected(occurrence)}
      >
        <td id='icon' style={{ color: `#${skull.color.toString(16).padStart(6, '0')}` }}>
          <Icon icon={skull.icon} />
        </td>
        <td>{skull.name}</td>
        <td>{occurrence.amount}</td>
        <td>{formatDate(new Date(occurrence.millis))}</td>
      </tr>
    );
  };

  return occurrences.map((o, i) => renderRow(i, o));
};

export const Summary = () => {
  const [start, setStart] = useState(EpochDays.today().subDays(7));
  const [end, setEnd] = useState(EpochDays.today());

  const filter = useMemo(
    () => {
      const effectiveEnd = end.addDays(1).getMillis();
      return (o: Occurrence) => o.millis.getTime() <= effectiveEnd;
    },
    [end],
  );

  const skulls = useSkulls();
  const occurrences = useOccurrences(start, filter);
  const edit = useEditOccurrence();
  const [selected, setSelected] = useState<Occurrence>();
  const [showFilters, setShowFilters] = useState(false);
  const [selectedSkulls, setSelectedSkulls] = useState<number[]>([]);

  const skullMap = useMemo(
    () => {
      return skulls.items.reduce((acc, curr) => {
        acc.set(curr.id, curr);
        return acc;
      }, new Map<number, Skull>());
    },
    [skulls],
  );

  useMemo(
    () => {
      occurrences.items.sort((a, b) => {
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
      return occurrences.items;
    },
    [occurrences.items],
  );

  const filteredOccurrences = useMemo(
    () => occurrences.items.filter(o => selectedSkulls.indexOf(o.skull) < 0),
    [occurrences.items, selectedSkulls],
  );

  const error = check.error(skulls, occurrences, edit);
  if (!!error) {
    return <Banner.Error error={error} />;
  }

  if (check.pending(skulls, occurrences, edit)) {
    return <Banner.Loading />;
  }

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
                      console.log(`Adding ${s.id}`);
                      selectedSkulls.push(s.id);
                    } else {
                      console.log(`Removing ${s.id}`);
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
      {(filteredOccurrences.length === 0)
        ? <Banner.Empty />
        : <table className='summary'>
          <tbody>
            <tr>
              <th id='icon' />
              <th>Name</th>
              <th>Amount</th>
              <th>Time</th>
            </tr>
            {renderRows(filteredOccurrences, skullMap, setSelected)}
          </tbody>
        </table>
      }
      {selected &&
        <Edit
          skull={skullMap.get(selected.skull)!}
          amount={selected.amount}
          millis={selected.millis}
          skulls={skulls.items}
          onAccept={occurrence => {
            edit.update({ ...occurrence, id: selected.id })
              .then(() => setSelected(undefined))
          }}
          onDelete={() => {
            edit.remove(selected)
              .then(() => setSelected(undefined))
          }}
          onCancel={() => setSelected(undefined)}
        />
      }
    </>
  );
}
