package main

import (
    "fmt"
    "os"
)

func main() {
	var input string
	if _, err := fmt.Scan(&input); err != nil {
	    fmt.Fprintf(os.Stderr, "failed to read input: %v\n", err)
	    os.Exit(1)
	}
	result, err := run(input)
	if err != nil {
	    fmt.Fprintf(os.Stderr, "run error: %v\n", err)
	    os.Exit(1)
	}
	fmt.Println(result)
}
