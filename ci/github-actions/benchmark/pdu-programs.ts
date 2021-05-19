export function getReleasedPduName<Version extends string>(version: Version) {
  return `pdu-${version}` as const
}
