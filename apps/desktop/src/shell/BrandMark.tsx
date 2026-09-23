import { useEffect, useRef, type JSX, type SVGProps } from 'react';
import mark from '../brand/mark.json';

export function BrandMark({
  size = 20,
  animated = false,
  processing = false,
  ...props
}: SVGProps<SVGSVGElement> & {
  size?: number;
  animated?: boolean;
  processing?: boolean;
}): JSX.Element {
  const svg = useRef<SVGSVGElement>(null);
  const blades = useRef<SVGGElement>(null);
  const processingRef = useRef(processing);
  const refreshSpeed = useRef<(() => void) | null>(null);

  useEffect(() => {
    const element = blades.current;
    if (!animated || !element?.animate || !window.matchMedia) return;
    const hoverTarget = svg.current?.parentElement;
    if (!hoverTarget) return;

    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
    let animation: Animation | undefined;
    let frame = 0;
    let hoverTimer: ReturnType<typeof setTimeout> | null = null;
    let previous = 0;
    let hoverBurst = false;
    const desiredSpeed = () => (processingRef.current ? 40 : hoverBurst ? 4 : 1);
    let speed = desiredSpeed();
    let target = speed;

    const easeSpeed = (now: number) => {
      const elapsed = previous ? Math.min(now - previous, 64) : 16;
      previous = now;
      speed += (target - speed) * (1 - Math.exp(-elapsed / 180));
      if (Math.abs(target - speed) < 0.005) speed = target;
      if (animation) animation.playbackRate = speed;
      frame = speed === target ? 0 : requestAnimationFrame(easeSpeed);
    };
    const run = () => {
      if (reducedMotion.matches) return;
      if (!animation) {
        animation = element.animate(
          [{ transform: 'rotate(0deg)' }, { transform: 'rotate(360deg)' }],
          { duration: 30_000, iterations: Infinity },
        );
      }
      previous = 0;
      animation.playbackRate = speed;
      animation.play();
      if (!frame && speed !== target) frame = requestAnimationFrame(easeSpeed);
    };
    const enter = () => {
      if (processingRef.current || reducedMotion.matches) return;
      hoverBurst = true;
      target = desiredSpeed();
      run();
      if (hoverTimer !== null) clearTimeout(hoverTimer);
      hoverTimer = setTimeout(() => {
        hoverTimer = null;
        hoverBurst = false;
        target = desiredSpeed();
        run();
      }, 1800);
    };
    const leave = () => {
      if (hoverTimer !== null) clearTimeout(hoverTimer);
      hoverTimer = null;
      hoverBurst = false;
      target = desiredSpeed();
      run();
    };
    const motionChanged = () => {
      if (reducedMotion.matches) {
        cancelAnimationFrame(frame);
        frame = 0;
        if (hoverTimer !== null) clearTimeout(hoverTimer);
        hoverTimer = null;
        hoverBurst = false;
        animation?.pause();
      } else {
        target = desiredSpeed();
        run();
      }
    };
    refreshSpeed.current = () => {
      if (processingRef.current && hoverTimer !== null) {
        clearTimeout(hoverTimer);
        hoverTimer = null;
        hoverBurst = false;
      }
      target = desiredSpeed();
      run();
    };
    hoverTarget.addEventListener('pointerenter', enter);
    hoverTarget.addEventListener('pointerleave', leave);
    reducedMotion.addEventListener('change', motionChanged);
    run();
    return () => {
      refreshSpeed.current = null;
      cancelAnimationFrame(frame);
      if (hoverTimer !== null) clearTimeout(hoverTimer);
      animation?.cancel();
      hoverTarget.removeEventListener('pointerenter', enter);
      hoverTarget.removeEventListener('pointerleave', leave);
      reducedMotion.removeEventListener('change', motionChanged);
    };
  }, [animated]);

  useEffect(() => {
    processingRef.current = processing;
    refreshSpeed.current?.();
  }, [processing]);

  return (
    <svg
      ref={svg}
      width={size}
      height={size}
      viewBox={mark.viewBox}
      fill="currentColor"
      aria-hidden="true"
      focusable="false"
      className="brand-mark"
      {...props}
    >
      <g ref={blades} className="brand-mark-blades">
        {[0, 90, 180, 270].map((angle) => (
          <path
            key={angle}
            d={mark.blade}
            transform={`rotate(${angle} 50 50)`}
            fillRule="evenodd"
          />
        ))}
      </g>
      <circle cx="50" cy="50" r={mark.hubRadius} />
    </svg>
  );
}
