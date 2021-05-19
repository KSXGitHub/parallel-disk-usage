export const STRICT_BASH_OPTIONS = ['-o', 'errexit', '-o', 'pipefail', '-o', 'nounset'] as const
export const STRICT_BASH = ['bash', ...STRICT_BASH_OPTIONS] as const
export default STRICT_BASH
