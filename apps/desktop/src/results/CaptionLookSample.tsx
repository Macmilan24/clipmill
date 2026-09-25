/** A small animated example that changes locally while looks are compared. */
import './captionLookSample.css';

export function CaptionLookSample({
  look,
  highlight,
}: {
  readonly look: 'clean' | 'minimal' | 'boxed';
  readonly highlight: boolean;
}) {
  return (
    <span
      className="caption-look-sample"
      data-look={look}
      data-highlight={highlight ? 'on' : 'off'}
      aria-hidden="true"
    >
      <span>Make</span> <span>moments</span> <span>matter</span>
    </span>
  );
}
