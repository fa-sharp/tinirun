import type { TinirunSchemas } from "@tinirun/client";

type Language = TinirunSchemas["CodeRunnerLanguage"];

export const LANGUAGES: { value: Language; label: string; color: string }[] = [
	{ value: "python", label: "Python", color: "#3b82f6" },
	{ value: "javascript", label: "JavaScript", color: "#eab308" },
	{ value: "typescript", label: "TypeScript", color: "#6366f1" },
	{ value: "go", label: "Go", color: "#06b6d4" },
	{ value: "rust", label: "Rust", color: "#f97316" },
	{ value: "bash", label: "Bash", color: "#22c55e" },
];

export const DEFAULT_CODE: Record<Language, string> = {
	python: `print("Hello from tinirun!")

for i in range(1, 6):
    print(f"  Count: {i}")
`,
	javascript: `console.log("Hello from tinirun!");

for (let i = 1; i <= 5; i++) {
    console.log(\`  Count: \${i}\`);
}
`,
	typescript: `const message: string = "Hello from tinirun!";
console.log(message);

const counts: number[] = [1, 2, 3, 4, 5];
for (const n of counts) {
    console.log(\`  Count: \${n}\`);
}
`,
	go: `package main

import "fmt"

func main() {
\tfmt.Println("Hello from tinirun!")
\tfor i := 1; i <= 5; i++ {
\t\tfmt.Printf("  Count: %d\\n", i)
\t}
}
`,
	rust: `fn main() {
    println!("Hello from tinirun!");
    for i in 1..=5 {
        println!("  Count: {}", i);
    }
}
`,
	bash: `echo "Hello from tinirun!"

for i in 1 2 3 4 5; do
    echo "  Count: $i"
done
`,
};

export const DEPS_PLACEHOLDER: Record<Language, string> = {
	python: "requests\nnumpy==1.26.0",
	javascript: "lodash\naxios@1.6.0",
	typescript: "zod",
	go: "github.com/gin-gonic/gin@v1.10.0",
	rust: "serde\nserde_json=1.0",
	bash: "jq",
};
