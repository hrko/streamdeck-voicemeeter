export const minimumAgeMs = 7 * 24 * 60 * 60 * 1000;

export function releaseAge(publishedAt, now = Date.now()) {
  const timestamp = Date.parse(publishedAt);
  if (!Number.isFinite(timestamp)) return "unknown";
  return now - timestamp >= minimumAgeMs ? "eligible" : "too-new";
}

export function selectVersion(metadata, current, now = Date.now()) {
  const major = current.split(".")[0];
  return Object.keys(metadata.versions).filter(version => /^\d+\.\d+\.\d+$/.test(version)
    && version.split(".")[0] === major && releaseAge(metadata.time?.[version], now) === "eligible")
    .sort((a, b) => {
      const av = a.split(".").map(Number), bv = b.split(".").map(Number);
      return bv[0] - av[0] || bv[1] - av[1] || bv[2] - av[2];
    })[0];
}
