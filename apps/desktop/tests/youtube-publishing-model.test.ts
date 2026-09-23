import { describe, expect, it } from 'vitest';
import { metadataLimits } from '../src/youtube/model.js';

describe('YouTube metadata transport parity', () => {
  it('counts title Unicode scalar characters, not UTF-16 code units', () => {
    expect(metadataLimits('😀'.repeat(100), '', []).problem).toBeNull();
    expect(metadataLimits('😀'.repeat(101), '', []).problem).toContain('1–100 characters');
  });
  it('bounds descriptions by UTF-8 bytes and permits ordinary line breaks', () => {
    expect(metadataLimits('Title', 'é'.repeat(2500), []).descriptionBytes).toBe(5000);
    expect(metadataLimits('Title', 'é'.repeat(2500), []).problem).toBeNull();
    expect(metadataLimits('Title', 'é'.repeat(2600), []).problem).toContain('5,000 UTF-8 bytes');
    expect(metadataLimits('Title', 'Line one\nLine two\tTabbed\rReturn', []).problem).toBeNull();
    expect(metadataLimits('Title', 'Hidden\u0085control', []).problem).not.toBeNull();
  });
  it('includes the transport’s separators and implicit tag quotes', () => {
    expect(metadataLimits('Title', '', ['two words']).tagBytes).toBe(12);
    expect(metadataLimits('Title', '', ['é'.repeat(249)]).tagBytes).toBe(499);
    expect(metadataLimits('Title', '', ['a'.repeat(497) + ' b']).problem).toContain(
      '500 UTF-8 bytes',
    );
  });
});
