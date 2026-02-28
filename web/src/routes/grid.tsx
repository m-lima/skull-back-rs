import * as Banner from '../components/banner';
import { Edit, Icon } from '../components/mod';
import { check, useSkulls, useQuicks, useOccurrences, Skull, Quick, EpochDays, useEditOccurrence } from '../store/mod';

import './grid.css';

import { useMemo, useState } from 'react';

const buildSkullButton = (
  quick: Quick,
  index: number,
  skullAmounts: Map<number, number> = new Map(),
  setSelected: (q: Quick) => void,
) => (
  <div
    key={index}
    className='grid-button grid-button-quick'
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

const newSkull = (skulls: Skull[], setSelected: (q: Quick) => void) => (
  <div
    className='grid-button'
    style={{ background: 'gray' }}
    onClick={() => setSelected({ skull: skulls[0], amount: 1 })}
  >
    <Icon icon='fas fa-plus' />
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

  if (skulls.items.length === 0) {
    return <Banner.NoSkulls />;
  }

  return (
    <>
      <div className='grid'>
        {
        quicks.items.length > 0
          ? quicks.items.map((q, i) => buildSkullButton(q, i, skullAmount, setSelected))
          : newSkull(skulls.items, setSelected)
        }
      </div>
      {selected && (
        <Edit
          skull={selected.skull}
          amount={selected.amount}
          skulls={skulls.items}
          onAccept={occurrence => {
            edit.create(occurrence)
              .then(() => {
                if (occurrences.items.length === 0) {
                  window.location.reload();
                }
                setSelected(undefined)
              })
          }}
          onCancel={() => setSelected(undefined)}
        />
      )}
    </>
  );
};
