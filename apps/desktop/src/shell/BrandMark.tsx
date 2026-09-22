import { useEffect, useRef, type JSX, type SVGProps } from 'react';
import mark from '../brand/mark.json';

export function BrandMark({
  size = 20,
  animated = false,
  ...props
}: SVGProps<SVGSVGElement> & { size?: number; animated?: boolean }): JSX.Element {
  const svg = useRef<SVGSVGElement>(null);
  const blades = useRef<SVGGElement>(null);

  useEffect(() => {
    const element = blades.current;
    if (!animated || !element?.animate || !window.matchMedia) return;
    const hoverTarget = svg.current?.parentElement;
    if (!hoverTarget) return;

    const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
    let animation: Animation | undefined;
    let frame = 0;
    let previous = 0;
    let speed = 1;
    let target = 1;
    let hovered = false;

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
      hovered = true;
      target = 2;
      run();
    };
    const leave = () => {
      hovered = false;
      target = 1;
      run();
    };
    const motionChanged = () => {
      if (reducedMotion.matches) {
        cancelAnimationFrame(frame);
        frame = 0;
        animation?.pause();
      } else {
        target = hovered ? 2 : 1;
        run();
      }
    };
    hoverTarget.addEventListener('pointerenter', enter);
    hoverTarget.addEventListener('pointerleave', leave);
    reducedMotion.addEventListener('change', motionChanged);
    run();
    return () => {
      cancelAnimationFrame(frame);
      animation?.cancel();
      hoverTarget.removeEventListener('pointerenter', enter);
      hoverTarget.removeEventListener('pointerleave', leave);
      reducedMotion.removeEventListener('change', motionChanged);
    };
  }, [animated]);

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
        {mark.paths.map((path) => (
          <path key={path} d={path} />
        ))}
      </g>
    </svg>
  );
}
