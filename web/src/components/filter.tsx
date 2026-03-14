import { Icon } from '../components/mod';
import { skullColor, EpochDays, Skull, Window } from '../store/mod';

import './filter.css';

import DatePicker from 'react-datepicker';
import { useState } from 'react';

interface FilterProps {
  skulls: Skull[];
  expanded: [boolean, (expanded: boolean) => void];
  start: [EpochDays, (start: EpochDays) => void];
  end: [EpochDays, (start: EpochDays) => void];
  selectedSkulls: [number[], (selected: number[]) => void];

  window?: [Window, (window: Window) => void];
  showLimits?: [boolean, (showLimits: boolean) => void];
}

export const Filter = (props: FilterProps) => {
  const [expanded, setExpanded] = props.expanded;
  const [start, setStart] = props.start;
  const [end, setEnd] = props.end;
  const [selectedSkulls, setSelectedSkulls] = props.selectedSkulls;

  const window = props.window && { value: props.window[0], set: props.window[1] };
  const showLimits = props.showLimits && { value: props.showLimits[0], set: props.showLimits[1] };

  const [windowStr, setWindowStr] = useState(window?.value.toString());

  return (
    <>
      <div className='filter-toggle' onClick={() => setExpanded(!expanded)}>
        <span id='label'>Filter</span>
        <Icon icon={expanded ? 'fas fa-caret-up' : 'fas fa-caret-down'} />
      </div>
      {expanded && (
        <>
          <div className='filter-inputs'>
            <div className='filter-input'>
              <b>Start</b>
              <DatePicker
                selected={new Date(start.getMillis())}
                dateFormat='dd/MM/yyyy'
                popperPlacement='bottom'
                onChange={d => d && setStart(new EpochDays(d))}
              />
            </div>
            <div className='filter-input'>
              <b>End</b>
              <DatePicker
                selected={new Date(end.getMillis())}
                dateFormat='dd/MM/yyyy'
                popperPlacement='bottom'
                onChange={d => d && setEnd(new EpochDays(d))}
              />
            </div>
            {window && (
              <div className='filter-input'>
                <b>Window</b>
                <input
                  type='text'
                  value={windowStr}
                  onChange={e => {
                    setWindowStr(e.target.value);
                    const parsed = Window.fromString(e.target.value);
                    if (parsed !== undefined) {
                      window.set(parsed);
                    }
                  }}
                  onBlur={() => setWindowStr(window.value.toString())}
                />
              </div>
            )}
          </div>
          <div className='filter-skulls'>
            {showLimits && (
              <div>
                <input
                  type='checkbox'
                  checked={showLimits.value}
                  onChange={() => showLimits.set(!showLimits.value)}
                />
                <label>Limits</label>
              </div>
            )}
            {props.skulls.map((s, i) => (
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
                <label htmlFor={s.name} style={{ color: skullColor(s) }}>
                  {s.name}
                </label>
              </div>
            ))}
          </div>
        </>
      )}
    </>
  );
};
