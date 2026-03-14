import './icon.css';

interface IconProps {
  icon: string;
  className?: string;
  margin?: boolean;
}

export const Icon = (props: IconProps) => {
  let newClass = props.icon;
  if (props.className) {
    newClass += ` ${props.className}`;
  }

  if (props.margin) {
    newClass += ' icon-with-margin';
  }

  return <i className={newClass} />;
};

interface SpinnerProps {
  className?: string;
  margin?: boolean;
}

export const Spinner = (props: SpinnerProps) =>
  Icon({ icon: 'fas fa-spinner icon-spinner', ...props });
