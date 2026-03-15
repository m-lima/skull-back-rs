import { Banner, Filter } from '../components/mod';
import {
  check,
  skullColor,
  useOccurrences,
  useSkulls,
  EpochDays,
  Occurrence,
  Window,
} from '../store/mod';

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
import { useMemo, useState } from 'react';
import * as datefns from 'date-fns';

ChartJS.register(LineElement, LinearScale, PointElement, TimeScale, Tooltip);

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
  beforeDraw: chart => {
    const {
      ctx,
      canvas,
      chartArea: { top, bottom, left, right },
      scales: { x },
    } = chart;

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
            : [a => datefns.subDays(a, 1), datefns.startOfDay];
    const minDay = new Date(x.min);

    let currDay = new Date(x.max);
    let prevDay = firstStep(currDay);
    let isDark = true;
    while (currDay > minDay) {
      const startX = x.getPixelForValue(prevDay.getTime());
      const endX = x.getPixelForValue(currDay.getTime());

      ctx.fillStyle = isDark ? dark : bright;
      ctx.fillRect(startX, top, endX - startX, bottom - top);

      currDay = prevDay;
      prevDay = stepper(prevDay);
      isDark = !isDark;
    }

    ctx.restore();
  },
};

export const Chart = () => {
  const [start, setStart] = useState(new EpochDays(datefns.subMonths(new Date(), 6)));
  const [end, setEnd] = useState(EpochDays.today());
  const [showFilter, setShowFilter] = useState(false);
  const [showLimits, setShowLimits] = useState(false);
  const [selectedSkulls, setSelectedSkulls] = useState<number[]>([]);

  const [window, setWindow] = useState(
    () => new Window({ amount: 15, unit: 'd' }, { amount: 1, unit: 'd' }, { amount: 1, unit: 'd' }),
  );

  const [windowLength, windowStep, windowBy] = useMemo(
    () => [window.getLength(), window.getStep(), window.getBy()],
    [window],
  );

  const effectiveStart = useMemo(() => start.getMillis() - windowLength, [start, windowLength]);
  const effectiveEnd = useMemo(() => end.addDays(1).getMillis(), [end]);

  const filter = useMemo(
    () => (o: Occurrence) => o.millis.getTime() <= effectiveEnd,
    [effectiveEnd],
  );

  const skulls = useSkulls();
  const occurrences = useOccurrences(effectiveStart, filter);

  const filteredOccurrences = useMemo(
    () => occurrences.items.filter(o => selectedSkulls.indexOf(o.skull) < 0),
    [occurrences.items, selectedSkulls],
  );

  const lineData = useMemo(() => {
    const cutPoint = filteredOccurrences.findIndex(o => o.millis.getTime() <= effectiveEnd);
    let occurrences = cutPoint < 0 ? [] : filteredOccurrences.slice(cutPoint);

    const datapoints = new Map<number, { millis: Date; amount: number }[]>();
    skulls.items
      .filter(s => selectedSkulls.find(id => id === s.id) === undefined)
      .forEach(s => datapoints.set(s.id, []));

    // Loop while
    // - there are more windows to explore
    // - there are more occurrences to explore
    // - we have occurrences that are inside the range
    for (
      let windowTop = effectiveEnd;
      windowTop >= start.getMillis() &&
      occurrences.length > 0 &&
      occurrences[0].millis.getTime() >= effectiveStart;
      windowTop -= windowStep
    ) {
      const label = new Date(windowTop);
      const windowBottom = windowTop - windowLength;

      datapoints.forEach((amounts, _) => amounts.push({ millis: label, amount: 0 }));

      let cutPoint = 0;
      for (let i = 0; i < occurrences.length; i++) {
        const occ = occurrences[i];
        const millis = occ.millis.getTime();
        // Remove the head of the array, so next iteration can skip these
        if (millis > windowTop - windowStep) {
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
          skull[skull.length - 1].amount += occ.amount * (windowBy / windowLength);
        } else {
          skull.push({ millis: label, amount: occ.amount });
        }
      }

      if (cutPoint > 0) {
        // TODO: Consider no `slice`ing and simply move the header forward. The array is dropped in the end, so we don't need to care
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
  }, [
    filteredOccurrences,
    selectedSkulls,
    showLimits,
    skulls,
    start,
    effectiveStart,
    effectiveEnd,
    windowLength,
    windowStep,
    windowBy,
  ]);

  const error = check.error(skulls, occurrences);
  if (error) {
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
        window={[window, setWindow]}
        showLimits={[showLimits, setShowLimits]}
      />
      {check.pending(skulls, occurrences) ? (
        <Banner.Loading />
      ) : filteredOccurrences.length === 0 ? (
        <Banner.Empty />
      ) : (
        <div style={{ flex: 1, minHeight: 0, minWidth: 0 }}>
          <Line
            options={options}
            data={{
              labels: [],
              datasets: lineData,
            }}
            plugins={[alternatingDaysPlugin]}
          />
        </div>
      )}
    </>
  );
};
