import { Banner, Edit, Filter, Icon } from '../components/mod';
import {
  check,
  skullColor,
  useEditOccurrence,
  useOccurrences,
  useSkulls,
  EpochDays,
  Occurrence,
  Skull,
} from '../store/mod';

import './summary.css';

import * as datefns from 'date-fns';
import { useMemo, useState } from 'react';

const renderRows = (
  occurrences: Occurrence[],
  skullMap: Map<number, Skull>,
  setSelected: (o: Occurrence) => void,
) => {
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
      <tr id={id ? 'bright' : 'dark'} key={index} onClick={() => setSelected(occurrence)}>
        <td id='icon' style={{ color: skullColor(skull) }}>
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
  const [selected, setSelected] = useState<Occurrence>();
  const [showFilter, setShowFilter] = useState(false);
  const [selectedSkulls, setSelectedSkulls] = useState<number[]>([]);

  const filter = useMemo(() => {
    const effectiveEnd = end.addDays(1).getMillis();
    return (o: Occurrence) => o.millis.getTime() <= effectiveEnd;
  }, [end]);

  const skulls = useSkulls();
  const occurrences = useOccurrences(start, filter);
  const edit = useEditOccurrence();

  const skullMap = useMemo(() => {
    return skulls.items.reduce((acc, curr) => {
      acc.set(curr.id, curr);
      return acc;
    }, new Map<number, Skull>());
  }, [skulls]);

  const filteredOccurrences = useMemo(
    () => occurrences.items.filter(o => selectedSkulls.indexOf(o.skull) < 0),
    [occurrences.items, selectedSkulls],
  );

  const error = check.error(skulls, occurrences, edit);
  if (!!error) {
    return <Banner.Error error={error} />;
  }

  return (
    <>
      <Filter
        skulls={skulls.items}
        expanded={[showFilter, setShowFilter]}
        start={[start, setStart]}
        end={[end, setEnd]}
        selectedSkulls={[selectedSkulls, setSelectedSkulls]}
      />
      {check.pending(skulls, occurrences, edit) ? (
        <Banner.Loading />
      ) : filteredOccurrences.length === 0 ? (
        <Banner.Empty />
      ) : (
        <table className='summary'>
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
      )}
      {selected && (
        <Edit
          skull={skullMap.get(selected.skull)!}
          amount={selected.amount}
          millis={selected.millis}
          skulls={skulls.items}
          onAccept={occurrence => {
            edit.update({ ...occurrence, id: selected.id }).then(() => setSelected(undefined));
          }}
          onDelete={() => {
            edit.remove(selected).then(() => setSelected(undefined));
          }}
          onCancel={() => setSelected(undefined)}
        />
      )}
    </>
  );
};
