let bin = "nur"
let os = ($env.OS | parse "{name}-{version}" | first)
let target = $env.TARGET
let format = $env.FORMAT
let src = $env.GITHUB_WORKSPACE
let version = (open Cargo.toml | get package.version)
let suffix = match $os.name {
    "windows" => ".exe"
    _ => ""
}
let target_path = $'target/($target)/release'
let release_bin = $'($target_path)/($bin)($suffix)'
let executables = $'($target_path)/($bin)*($suffix)'
let dest = $'($bin)-($version)-($target)'
let dist = $'($env.GITHUB_WORKSPACE)/output'

def 'hr-line' [] {
    print $'(ansi g)----------------------------------------------------------------------------(ansi reset)'
}

print $'Config for this run is:'
{
    bin: $bin
    os: $os
    target: $target
    format: $format
    src: $src
    version: $version
    suffix: $suffix
    target_path: $target_path
    release_bin: $release_bin
    executables: $executables
    dest: $dest
    dist: $dist
} | table --expand | print

print $'Packaging ($bin) v($version) for ($target) in ($src)...'

hr-line
print $'Preparing build dependencies for ($bin)...'
match [$os.name, $format, $target] {
    ["ubuntu", "bin", "aarch64-unknown-linux-gnu"] => {
        sudo apt update
        sudo apt install -y gcc-aarch64-linux-gnu
        $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = 'aarch64-linux-gnu-gcc'
    }
    ["windows", "msi", _] => {
        cargo install cargo-wix
    }
}

hr-line
print $'Start building ($bin)...'
match [$os.name, $format] {
    ["windows", _] => {
        cargo build --release --all --target $target
    }
    [_, "bin"] => {
        cargo build --release --all --target $target --features=static-link-openssl
    }
}

hr-line
print $'Check ($bin) version...'
let built_version = do --ignore-errors { ^$release_bin --version } | str join
if ($built_version | str trim | is-empty) {
    print $'(ansi r)Incompatible arch: cannot run ($release_bin)(ansi reset)'
} else {
    print $" -> built version is: ($built_version)"
}

hr-line
print $'Cleanup release target path...'
rm -rf ...(glob $'($target_path)/*.d')

hr-line
print $'Copying ($bin) and other release files to ($dest)...'
match [$os.name, $format] {
    ["windows", "msi"] => {
        print ' -> skipping for MSI build'
    }
    _ => {
        mkdir $dest
        [README.md LICENSE ...(glob $executables)] | each {|it| cp -rv $it $dest } | flatten
    }
}

hr-line
print $'Creating release archive in ($dist)...'
mkdir $dist
mut archive = $'($dist)/($dest).tar.gz'
match [$os.name, $format] {
    ["windows", "msi"] => {
        $archive = $'($dist)/($dest).msi'
        let main_version = if ("+" in $version) {$version | split row "+" | get 0} else {$version}
        cargo wix --no-build --nocapture --target $target --package $bin --output $archive --install-version $main_version
    }
    ["windows", "bin"] => {
        $archive = $'($dist)/($dest).zip'
        7z a $archive $dest
    }
    _ => {
        tar -czf $archive $dest
    }
}
print $' -> archive: ($archive)'

hr-line
print $'Provide archive to GitHub...'
match $os.name {
    "windows" => {
        # Workaround for https://github.com/softprops/action-gh-release/issues/280
        echo $"archive=($archive | str replace --all '\' '/')" | save --append $env.GITHUB_OUTPUT
    }
    _ => {
        echo $"archive=($archive)" | save --append $env.GITHUB_OUTPUT
    }
}
