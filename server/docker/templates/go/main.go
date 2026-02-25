package main

import (
    "bufio"
    "fmt"
    "os"
    "strings"
)

func main() {
	var lines []string
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		lines = append(lines, scanner.Text())
	}
	if err := scanner.Err(); err != nil {
		fmt.Fprintf(os.Stderr, "failed to read input: %v\n", err)
		os.Exit(1)
	}
	input := strings.Join(lines, "\n")

	result, err := run(input)
	if err != nil {
	    fmt.Fprintf(os.Stderr, "run error: %v\n", err)
	    os.Exit(1)
	}
	fmt.Println(result)
}
