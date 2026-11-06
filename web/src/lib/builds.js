// The downloads of one platform: x64 and Arm64 as two entries when both exist,
// otherwise the single build, without an architecture label.
export const builds = (x64, arm64) => (arm64 ? [{ arch: 'x64', ...x64 }, { arch: 'Arm64', ...arm64 }] : [x64])
