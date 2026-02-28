import * as Banner from '../components/banner';
import { Edit, Icon } from '../components/mod';
import { check, useSkulls, useQuicks, useOccurrences, Quick, EpochDays, useEditOccurrence } from '../store/mod';

import './grid.css';

import { useMemo, useState } from 'react';


// TODO: Make the 12 most common combinations in recent history
// WITH
//   last AS (
//     SELECT
//       MAX(millis) AS max
//     FROM
//       occurrences
//   ),
//   scored AS (
//     SELECT
//       o.skull,
//       o.amount,
//       COUNT(*) AS count,
//       SUM(POWER(0.5, (l.max - o.millis) / 864000000.0)) AS score
//     FROM
//       occurrences o
//     CROSS JOIN
//       last l
//     GROUP BY
//       o.skull,
//       o.amount
//     ORDER BY
//       score DESC
//     LIMIT
//       12
//   )
// SELECT
//   *
// FROM
//   scored
// LEFT JOIN
//   skulls
// ON
//   skull = skulls.id
// with recent as (select skull, amount from occurrences order by millis desc limit 1000), quicks as (select skull, amount, count(*) count from recent group by skull, amount order by count desc limit 12) select * from quicks left join skulls on quicks.skull = skulls.id
// with new as (select skull, amount from occurrences order by millis desc limit 100), mid as (select skull, amount from occurrences order by millis desc limit 1000), old as (select skull, amount from occurrences order by millis desc limit 10000), combined as (select * from new union all select * from mid union all select * from old), quicks as (select skull, amount, count(*) count from combined group by skull, amount order by count desc limit 12) select * from quicks left join skulls on quicks.skull = skulls.id

const buildSkullButton = (
  quick: Quick,
  index: number,
  skullAmounts: Map<number, number> = new Map(),
  setSelected: (q: Quick) => void,
) => (
  <div
    key={index}
    className='grid-button'
    title={
      `Skull: ${quick.skull.name}\nAmount: ${quick.amount}` +
      (!!quick.skull.limit ? `\nLimit: ${quick.skull.limit}` : '')
    }
    style={{ background: `#${quick.skull.color.toString(16).padStart(6, '0')}` }}
    onClick={() => setSelected(quick)}
  >
    <Icon icon={quick.skull.icon} />
    <div
      className='grid-button-amount'
      id={idForQuick(skullAmounts, quick)}
    >
      {quick.amount}
    </div>
  </div >
);

const idForQuick = (
  skullAmounts: Map<number, number>,
  quick: Quick,
) => {
  if (quick.skull.limit && skullAmounts.has(quick.skull.id)) {
    const skullAmount = skullAmounts.get(quick.skull.id)! + quick.amount;
    if (skullAmount >= quick.skull.limit) {
      return 'grid-button-over-limit';
    } else if (skullAmount >= quick.skull.limit * 0.75) {
      return 'grid-button-near-limit';
    } else {
      return undefined;
    }
  } else {
    return undefined;
  }
};

export const Grid = () => {
  const skulls = useSkulls();
  const quicks = useQuicks();
  const occurrences = useOccurrences(EpochDays.today().subDays(1));
  const edit = useEditOccurrence();
  const [selected, setSelected] = useState<Quick>();

  const skullAmount = useMemo(() => {
    return occurrences.items
      .reduce((acc, curr) => {
        let amount = acc.get(curr.skull);
        if (!!amount) {
          amount += curr.amount;
        } else {
          amount = curr.amount;
        }
        acc.set(curr.skull, amount);
        return acc;
      }, new Map<number, number>());
  }, [occurrences.items]);

  const error = check.error(skulls, quicks, occurrences, edit);
  if (!!error) {
    return <Banner.Error error={error} />;
  }

  if (check.pending(skulls, quicks, occurrences, edit)) {
    return <Banner.Loading />;
  }

  if (quicks.items.length === 0) {
    return <Banner.NoQuicks />;
  }

  return (
    <>
      <div className='grid'>
        {quicks.items.map((q, i) => buildSkullButton(q, i, skullAmount, setSelected))}
      </div>
      {selected && (
        <Edit
          skull={selected.skull}
          amount={selected.amount}
          skulls={skulls.items}
          onAccept={occurrence => {
            edit.create(occurrence)
              .then(() => setSelected(undefined))
          }}
          onCancel={() => setSelected(undefined)}
        />
      )}
    </>
  );
};
