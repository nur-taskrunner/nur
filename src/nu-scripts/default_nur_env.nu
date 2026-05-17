# nur Environment File

# Directories to search for scripts when calling source or use
# The default for this is $nur.default-lib-dir which is $nur-project-path/.nur/scripts
$env.NU_LIB_DIRS = [
    $nur.default-lib-dir
]

# Ensure indicator is set.
$env.PROMPT_INDICATOR = "> "

# To load from a custom file you can use:
# source ($nur.project-path | path join 'custom.nu')
