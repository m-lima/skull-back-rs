import { Icon } from './icon';
import { ProtoOccurrence, Skull } from '../store/mod';

import './edit.css';

import DatePicker from 'react-datepicker';
import 'react-datepicker/dist/react-datepicker.css';
import { useRef, useState } from 'react';

interface EditProps {
  skull: Skull,
  amount: number,
  millis?: Date,
  skulls: Skull[],
  onAccept: (value: ProtoOccurrence) => void;
  onDelete?: () => void;
  onCancel: () => void;
}

interface StagedOccurrence {
  skull: Skull,
  amount: string,
  millis: Date,
}

export const Edit = (props: EditProps) => {
  const [markedForDeletion, setMarkedForDeletion] = useState(false);
  const [stagedValue, setStagedValue] = useState<StagedOccurrence>({
    skull: props.skull,
    amount: String(props.amount),
    millis: props.millis ? props.millis : new Date(),
  });
  const amountRef = useRef<HTMLInputElement>(null);

  const stageSkull = (skull: string | number) => {
    const maybeSkull = typeof skull === 'string'
      ? props.skulls.find(s => s.name === skull)
      : props.skulls.find(s => s.id === skull);
    if (!!maybeSkull) {
      setStagedValue({
        ...stagedValue,
        skull: maybeSkull,
      });
    }
  };

  const stageAmount = (amount: string) => {
    setStagedValue({
      ...stagedValue,
      amount: amount.replace(',', '.'),
    });
  }

  const stageMillis = (millis: Date | null) => {
    if (!!millis) {
      setStagedValue({
        ...stagedValue,
        millis: millis,
      });
    }
  }

  const commit = () => {
    if (markedForDeletion) {
      props.onDelete!();
    } else {
      const amount = Number(stagedValue.amount);
      if (!amount && amountRef.current) {
        amountRef.current.focus();
        return;
      }

      props.onAccept({
        skull: stagedValue.skull.id,
        amount,
        millis: stagedValue.millis,
      });
    }
  };

  return (
    <div className='edit'>
      <div className='edit-container'>
        <div className='edit-inputs' id={markedForDeletion ? 'delete' : ''}>
          <div className='edit-input'>
            <b>Type</b>
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
          <div className='edit-input'>
            <b>Amount</b>
            <input
              id={Number(stagedValue.amount) ? '' : 'invalid'}
              ref={amountRef}
              disabled={markedForDeletion}
              type='text'
              inputMode='decimal'
              min={0}
              step={0.1}
              value={stagedValue.amount}
              onChange={e => stageAmount(e.target.value)}
            />
          </div>
          <div className='edit-input'>
            <b>Time</b>
            <DatePicker
              disabled={markedForDeletion}
              selected={stagedValue.millis}
              showTimeSelect
              dateFormat='dd/MM/yyyy HH:mm'
              timeIntervals={15}
              popperPlacement='top'
              onChange={d => stageMillis(d)}
            />
          </div>
          {!!props.onDelete && (
            <div className='edit-input'>
              <div
                className='edit-delete'
                onClick={() => setMarkedForDeletion(!markedForDeletion)}
              >
                <Icon icon='fas fa-trash' />
              </div>
            </div>
          )}
        </div>
        <div className='edit-buttons'>
          <div className='icon-button' id='accept' title='Accept' onClick={commit}>
            <Icon icon='fas fa-check' />
          </div>
          <div className='icon-button' id='cancel' title='Cancel' onClick={props.onCancel}>
            <Icon icon='fas fa-times' />
          </div>
        </div>
      </div>
    </div>
  );
}
