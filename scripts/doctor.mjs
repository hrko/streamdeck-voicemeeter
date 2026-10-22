import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const expected = JSON.parse(readFileSync('package.json', 'utf8')).packageManager.split('@').at(-1);
function output(command, args) {
  const result = spawnSync(command, args, { encoding: 'utf8', shell: false });
  if (result.status !== 0) throw new Error(`${command}: ${result.error ?? result.stderr}`);
  return result.stdout.trim();
}
if (process.platform !== 'win32' || process.arch !== 'x64') throw new Error('The supported build host is Windows x64.');
const version = output('pnpm.exe', ['--version']);
if (version !== expected) throw new Error(`pnpm ${version} differs from packageManager ${expected}`);
console.log(output('rustc.exe', ['--version']));
console.log(`Node ${process.version}; pnpm ${version}`);
const vswhere = join(process.env['ProgramFiles(x86)'] ?? '', 'Microsoft Visual Studio', 'Installer', 'vswhere.exe');
if (!existsSync(vswhere)) throw new Error('Install Visual Studio Build Tools with the Desktop development with C++ workload.');
const install = output(vswhere, ['-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-property', 'installationPath']);
if (!install) throw new Error('MSVC x64 build tools are missing.');
const sdkRoot = join(process.env['ProgramFiles(x86)'] ?? '', 'Windows Kits', '10', 'Lib');
if (!existsSync(sdkRoot)) throw new Error('Install the Windows 10 or 11 SDK.');
console.log(`MSVC: ${install}\nWindows SDK: ${sdkRoot}`);
