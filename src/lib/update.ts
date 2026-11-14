import type { UpdateInfo } from "../types";

const LATEST_RELEASE_API =
  "https://api.github.com/repos/luigihenrick/ClipboardHistory/releases/latest";

interface GitHubRelease {
  tag_name: string;
  name?: string;
  body?: string;
  html_url: string;
}

interface Version {
  major: number;
  minor: number;
  patch: number;
  prerelease: string | null;
}

function parseVersion(value: string): Version | null {
  const match = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/.exec(
    value.trim(),
  );
  if (!match) return null;

  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
    prerelease: match[4] ?? null,
  };
}

function comparePrerelease(remote: string, current: string): number {
  const remoteParts = remote.split(".");
  const currentParts = current.split(".");

  for (let index = 0; index < Math.max(remoteParts.length, currentParts.length); index += 1) {
    const remotePart = remoteParts[index];
    const currentPart = currentParts[index];

    if (remotePart === undefined) return -1;
    if (currentPart === undefined) return 1;
    if (remotePart === currentPart) continue;

    const remoteIsNumber = /^\d+$/.test(remotePart);
    const currentIsNumber = /^\d+$/.test(currentPart);

    if (remoteIsNumber && currentIsNumber) {
      return Number(remotePart) > Number(currentPart) ? 1 : -1;
    }
    if (remoteIsNumber) return -1;
    if (currentIsNumber) return 1;
    return remotePart > currentPart ? 1 : -1;
  }

  return 0;
}

/** Returns true when `remote` is a valid version newer than `current`. */
export function isNewerVersion(remote: string, current: string): boolean {
  const remoteVersion = parseVersion(remote);
  const currentVersion = parseVersion(current);
  if (!remoteVersion || !currentVersion) return false;

  if (remoteVersion.major !== currentVersion.major) {
    return remoteVersion.major > currentVersion.major;
  }
  if (remoteVersion.minor !== currentVersion.minor) {
    return remoteVersion.minor > currentVersion.minor;
  }
  if (remoteVersion.patch !== currentVersion.patch) {
    return remoteVersion.patch > currentVersion.patch;
  }

  if (remoteVersion.prerelease === currentVersion.prerelease) return false;
  if (!remoteVersion.prerelease) return true;
  if (!currentVersion.prerelease) return false;
  return comparePrerelease(remoteVersion.prerelease, currentVersion.prerelease) > 0;
}

function updateFromRelease(release: unknown, current: string): UpdateInfo | null {
  if (!release || typeof release !== "object") {
    throw new Error("Invalid update response.");
  }

  const data = release as Partial<GitHubRelease>;
  if (typeof data.tag_name !== "string" || typeof data.html_url !== "string") {
    throw new Error("Invalid update response.");
  }
  if (!data.html_url.startsWith("https://") || !isNewerVersion(data.tag_name, current)) {
    return null;
  }

  return {
    version: data.tag_name,
    name: typeof data.name === "string" ? data.name : data.tag_name,
    notes: typeof data.body === "string" ? data.body : "",
    url: data.html_url,
  };
}

/** Checks GitHub for a release newer than `current` using the webview's HTTP stack. */
export async function checkForUpdate(
  current: string,
  fetcher: typeof fetch = fetch,
): Promise<UpdateInfo | null> {
  const response = await fetcher(LATEST_RELEASE_API, {
    headers: { Accept: "application/vnd.github+json" },
  });

  if (!response.ok) {
    throw new Error(`Update check failed with status ${response.status}`);
  }

  return updateFromRelease(await response.json(), current);
}
