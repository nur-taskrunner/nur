def "nu-complete nur task-names" [] {
  ^nur --list | lines
}

# nur - a taskrunner based on nu shell.
export extern nur [
  --help(-h)  # Display the help message for this command
  --version(-v)  # Output version number and exit
  --list(-l)  # List available tasks and then just exit
  --nurfile(-f)  # Define which nurfile to search and load (defaults to 'nurfile')
  --quiet(-q)  # Do not output anything but what the task produces
  --stdin  # Attach stdin to called nur task
  --commands(-c)  # Run the given commands after nurfiles have been loaded
  --enter-shell  # Enter a nu REPL shell after the nurfiles have been loaded (use only for debugging)
  task_name?: string@"nu-complete nur task-names"  # Name of the task to run (optional)
  ...args  # Parameters to the executed task
]
