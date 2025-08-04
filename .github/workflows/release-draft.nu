# aggregate a list of artifacts used for release assets
let files = ls artifacts | get name
if (($files | length) == 0) {
    error make {
        msg: "No build artifacts found. Release assets are required."
    }
}
print "The following files would be used as release assets:"
print $files

# Check the tag to be used for the drafted release.
# The tag used for GitHub releases must match the version published to crates.io.
# Otherwise, cargo-binstall fails to find the release from which to download standalone binaries.
let ref = $env.GITHUB_REF
if (not ($ref | str starts-with 'refs/tags/')) {
    print $"Not drafting release for ref ($ref) because it is not a tag."
} else {
    let version = open Cargo.toml | get package.version
    let tag = $ref | str substring 10..
    if ($version == ($tag | str trim --left --char 'v')) {
        let main_version = if ("+" in $version) {$version | split row "+" | get 0} else {$version}
        print $"The tag ($tag) will be used to draft a release for version ($main_version)."
        # Use gh-cli tool to draft a release on GitHub.
        # This command uses the current `git checkout` to get repo info.
        ^gh release create $tag --draft --generate-notes --title $'Release ($main_version)' ...$files
    } else {
        error make {
            msg: $"The tag ($tag) does not match the package version ($version)"
        }
    }
}
