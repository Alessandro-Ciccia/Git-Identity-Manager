export type NavigationSection = 'overview' | 'profiles' | 'repositories' | 'rules' | 'settings';

export type NavigationItem = {
  id: NavigationSection;
  label: string;
  description: string;
};

export const navigationItems = [
  {
    id: 'overview',
    label: 'Overview',
    description: 'Accounts, repositories, and identity health at a glance.',
  },
  {
    id: 'profiles',
    label: 'Profiles',
    description: 'Reusable Git identities without authentication secrets.',
  },
  {
    id: 'repositories',
    label: 'Repositories',
    description: 'Registered repositories and their effective Git identities.',
  },
  {
    id: 'rules',
    label: 'Rules',
    description: 'Optional directory-based identity automation.',
  },
  {
    id: 'settings',
    label: 'Settings',
    description: 'Local application preferences and diagnostics.',
  },
] as const satisfies readonly NavigationItem[];

export function isNavigationSection(value: string): value is NavigationSection {
  return navigationItems.some((item) => item.id === value);
}
