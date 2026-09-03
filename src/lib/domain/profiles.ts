import type { GithubAccount } from './github';

export type GithubAccountReference = {
  hostname: string;
  username: string;
};

export type GitProfile = {
  id: string;
  label: string;
  gitName: string;
  gitEmail: string;
  githubAccount: GithubAccountReference | null;
};

export type ProfileInput = Omit<GitProfile, 'id'>;

export type ProfileDraft = {
  label: string;
  gitName: string;
  gitEmail: string;
  githubAccountKey: string;
};

export type ProfileValidationErrors = Partial<
  Record<'label' | 'gitName' | 'gitEmail' | 'githubAccount', string>
>;

export const emptyProfileDraft: ProfileDraft = {
  label: '',
  gitName: '',
  gitEmail: '',
  githubAccountKey: '',
};

export function githubReferenceKey(reference: GithubAccountReference): string {
  return `${reference.hostname.toLowerCase()}::${reference.username.toLowerCase()}`;
}

export function profileDraft(profile: GitProfile): ProfileDraft {
  return {
    label: profile.label,
    gitName: profile.gitName,
    gitEmail: profile.gitEmail,
    githubAccountKey: profile.githubAccount ? githubReferenceKey(profile.githubAccount) : '',
  };
}

export function validateProfileDraft(draft: ProfileDraft): ProfileValidationErrors {
  const errors: ProfileValidationErrors = {};
  const label = draft.label.trim();
  const gitName = draft.gitName.trim();
  const gitEmail = draft.gitEmail.trim();

  if (!validText(label, 80)) {
    errors.label = 'Enter a label between 1 and 80 characters.';
  }
  if (!validText(gitName, 200)) {
    errors.gitName = 'Enter a Git name between 1 and 200 characters.';
  }
  if (!validEmail(gitEmail)) {
    errors.gitEmail = 'Enter a valid Git email address.';
  }
  if (draft.githubAccountKey && !parseGithubReferenceKey(draft.githubAccountKey)) {
    errors.githubAccount = 'Select a valid GitHub account.';
  }

  return errors;
}

export function hasProfileValidationErrors(errors: ProfileValidationErrors): boolean {
  return Object.keys(errors).length > 0;
}

export function profileInputFromDraft(draft: ProfileDraft): ProfileInput | null {
  if (hasProfileValidationErrors(validateProfileDraft(draft))) {
    return null;
  }

  return {
    label: draft.label.trim(),
    gitName: draft.gitName.trim(),
    gitEmail: draft.gitEmail.trim(),
    githubAccount: draft.githubAccountKey ? parseGithubReferenceKey(draft.githubAccountKey) : null,
  };
}

export function accountReference(account: GithubAccount): GithubAccountReference {
  return { hostname: account.hostname, username: account.username };
}

export function findAssociatedAccount(
  reference: GithubAccountReference,
  accounts: readonly GithubAccount[],
): GithubAccount | undefined {
  const key = githubReferenceKey(reference);
  return accounts.find((account) => githubReferenceKey(accountReference(account)) === key);
}

function validText(value: string, maxLength: number): boolean {
  return value.length > 0 && Array.from(value).length <= maxLength && !hasControlCharacter(value);
}

function hasControlCharacter(value: string): boolean {
  return Array.from(value).some((character) => {
    const codePoint = character.codePointAt(0) ?? 0;
    return codePoint <= 31 || (codePoint >= 127 && codePoint <= 159);
  });
}

function validEmail(email: string): boolean {
  if (!email || Array.from(email).length > 254 || /\s/.test(email) || hasControlCharacter(email)) {
    return false;
  }

  const separator = email.indexOf('@');
  if (separator <= 0 || separator !== email.lastIndexOf('@')) {
    return false;
  }

  const local = email.slice(0, separator);
  const domain = email.slice(separator + 1);
  return (
    local.length <= 64 &&
    !local.startsWith('.') &&
    !local.endsWith('.') &&
    !local.includes('..') &&
    domain.includes('.') &&
    validHostname(domain)
  );
}

function parseGithubReferenceKey(value: string): GithubAccountReference | null {
  const separator = value.indexOf('::');
  if (separator <= 0 || separator !== value.lastIndexOf('::')) {
    return null;
  }

  const hostname = value.slice(0, separator).trim().toLowerCase();
  const username = value.slice(separator + 2).trim();
  if (!validHostname(hostname) || !validUsername(username)) {
    return null;
  }

  return { hostname, username };
}

function validHostname(hostname: string): boolean {
  if (
    !hostname ||
    hostname.length > 253 ||
    !Array.from(hostname).every((character) => (character.codePointAt(0) ?? 128) <= 127) ||
    hostname.startsWith('.') ||
    hostname.endsWith('.')
  ) {
    return false;
  }

  return hostname
    .split('.')
    .every(
      (label) =>
        label.length > 0 &&
        label.length <= 63 &&
        /^[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?$/.test(label),
    );
}

function validUsername(username: string): boolean {
  return (
    username.length > 0 &&
    username.length <= 100 &&
    /^[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?$/.test(username)
  );
}
