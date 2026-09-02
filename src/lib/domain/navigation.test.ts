import { describe, expect, it } from 'vitest';

import { isNavigationSection, navigationItems } from './navigation';

describe('navigation domain', () => {
  it('contains the product shell sections in order', () => {
    expect(navigationItems.map((item) => item.id)).toEqual([
      'overview',
      'profiles',
      'repositories',
      'rules',
      'settings',
    ]);
  });

  it('validates navigation section ids', () => {
    expect(isNavigationSection('profiles')).toBe(true);
    expect(isNavigationSection('shell')).toBe(false);
  });
});
