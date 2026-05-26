/**
 * Escape a value for use inside a PowerShell single-quoted string literal.
 */
export function escapePowerShellSingleQuoted(value) {
  return value.replace(/\\/g, "\\\\").replace(/'/g, "''");
}
