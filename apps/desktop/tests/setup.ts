/**
 * One rendered tree per test.
 *
 * Testing Library unmounts automatically only when Vitest's globals are on, and
 * they are off here — so without this every render would pile into the same
 * document and the second test to look for a project title would find two.
 */
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

afterEach(cleanup);
