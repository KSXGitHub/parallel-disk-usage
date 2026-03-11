export function getReleasedPduName<Version extends string>(version: Version) {
  return `pdu-${version}` as const
}

export function getReleasedPduMuslName<Version extends string>(version: Version) {
  return `pdu-${version}-musl` as const
}
