import type { TinirunSchemas } from "@tinirun/client";
import {
	type EventSourceMessage,
	EventSourceParserStream,
} from "eventsource-parser/stream";
import {
	ChevronDown,
	ChevronUp,
	Loader2,
	PackagePlus,
	Play,
	Trash2,
} from "lucide-react";
import React, {
	Suspense,
	useCallback,
	useEffect,
	useRef,
	useState,
} from "react";
import { runCodeSnippetServerFn } from "@/api/runCode";

const CodeEditor = React.lazy(() => import("./CodeEditor"));

type Language = TinirunSchemas["CodeRunnerLanguage"];
type Chunk = TinirunSchemas["CodeRunnerChunk"];

interface OutputLine {
	id: number;
	chunk: Chunk;
}

const LANGUAGES: { value: Language; label: string; color: string }[] = [
	{ value: "python", label: "Python", color: "#3b82f6" },
	{ value: "javascript", label: "JavaScript", color: "#eab308" },
	{ value: "typescript", label: "TypeScript", color: "#6366f1" },
	{ value: "go", label: "Go", color: "#06b6d4" },
	{ value: "rust", label: "Rust", color: "#f97316" },
	{ value: "bash", label: "Bash", color: "#22c55e" },
];

const DEFAULT_CODE: Record<Language, string> = {
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

const DEPS_PLACEHOLDER: Record<Language, string> = {
	python: "requests\nnumpy==1.26.0",
	javascript: "lodash\naxios@1.6.0",
	typescript: "zod\n@types/node",
	go: "github.com/gin-gonic/gin@v1.10.0",
	rust: 'serde = "1.0"\ntokio = { version = "1", features = ["full"] }',
	bash: "",
};

let nextId = 0;

function ChunkLine({ chunk }: { chunk: Chunk }) {
	if (chunk.event === "stdout") {
		return (
			<span className="text-zinc-100 whitespace-pre-wrap wrap-break-word">
				{chunk.data}
			</span>
		);
	}
	if (chunk.event === "stderr") {
		return (
			<span className="text-amber-400 whitespace-pre-wrap wrap-break-word">
				{chunk.data}
			</span>
		);
	}
	if (chunk.event === "info") {
		return (
			<span className="text-sky-500 whitespace-pre-wrap wrap-break-word">
				<span className="opacity-60 select-none text-xs mr-1">[info]</span>
				{chunk.data}
			</span>
		);
	}
	if (chunk.event === "debug") {
		return (
			<span className="text-zinc-500 whitespace-pre-wrap wrap-break-word">
				<span className="opacity-60 select-none text-xs mr-1">[debug]</span>
				{chunk.data}
			</span>
		);
	}
	if (chunk.event === "error") {
		const err = chunk.data;
		const detail = "logs" in err && err.logs ? `\n${err.logs}` : "";
		return (
			<span className="text-red-400 whitespace-pre-wrap wrap-break-word">
				<span className="font-semibold">✗ Error:</span> {err.message}
				{detail}
			</span>
		);
	}
	if (chunk.event === "result") {
		const r = chunk.data;
		const exited = r.exit_code != null;
		const success = exited && r.exit_code === 0 && !r.timeout;
		const label = r.timeout
			? "⏱ Timed out"
			: success
				? "✓ Exited 0"
				: `✗ Exited ${r.exit_code ?? "?"}`;
		return (
			<span
				className={`font-semibold whitespace-pre-wrap wrap-break-word ${
					r.timeout
						? "text-amber-400"
						: success
							? "text-emerald-400"
							: "text-red-400"
				}`}
			>
				{label}
			</span>
		);
	}
	return null;
}

export function CodeRunner() {
	const [language, setLanguage] = useState<Language>("python");
	const [code, setCode] = useState(DEFAULT_CODE.python);
	const [dependencies, setDependencies] = useState("");
	const [showDeps, setShowDeps] = useState(false);
	const [isRunning, setIsRunning] = useState(false);
	const [outputLines, setOutputLines] = useState<OutputLine[]>([]);
	const [hasRun, setHasRun] = useState(false);
	const outputRef = useRef<HTMLDivElement>(null);
	const readerRef =
		useRef<ReadableStreamDefaultReader<EventSourceMessage> | null>(null);

	const handleLanguageChange = useCallback((lang: Language) => {
		setLanguage(lang);
		setCode(DEFAULT_CODE[lang]);
		setDependencies("");
	}, []);

	useEffect(() => {
		const el = outputRef.current;
		if (el && outputLines.length > 0) {
			el.scrollTop = el.scrollHeight;
		}
	}, [outputLines.length]);

	const handleRun = async () => {
		if (isRunning) {
			await readerRef.current?.cancel();
			readerRef.current = null;
			setIsRunning(false);
			return;
		}

		const deps = dependencies
			.split("\n")
			.map((d) => d.trim())
			.filter(Boolean);

		setIsRunning(true);
		setHasRun(true);
		setOutputLines([]);

		try {
			const res = await runCodeSnippetServerFn({
				data: {
					code,
					lang: language,
					dependencies: deps.length > 0 ? deps : undefined,
					timeout: 60,
					mem_limit_mb: 256,
					cpu_limit: 0.5,
				},
			});
			if (!res.body) throw new Error("No response body");
			const reader = res.body
				.pipeThrough(new TextDecoderStream())
				.pipeThrough(new EventSourceParserStream())
				.getReader();
			readerRef.current = reader;

			while (true) {
				const { done, value } = await reader.read();
				if (done) break;
				setOutputLines((prev) => [
					...prev,
					{ id: nextId++, chunk: JSON.parse(value.data) },
				]);
			}
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			setOutputLines((prev) => [
				...prev,
				{
					id: nextId++,
					chunk: {
						event: "error",
						data: { error: "docker", message },
					},
				},
			]);
		} finally {
			readerRef.current = null;
			setIsRunning(false);
		}
	};

	const handleClear = () => {
		setOutputLines([]);
		setHasRun(false);
	};

	const activeLang =
		LANGUAGES.find((l) => l.value === language) ?? LANGUAGES[0];

	return (
		<div className="flex flex-col h-full overflow-hidden">
			{/* Language selector bar */}
			<div className="flex items-center gap-1 px-3 py-2 bg-zinc-900 border-b border-zinc-800 shrink-0 overflow-x-auto">
				{LANGUAGES.map((lang) => {
					const isActive = lang.value === language;
					return (
						<button
							key={lang.value}
							type="button"
							onClick={() => handleLanguageChange(lang.value)}
							className={`px-3 py-1.5 rounded-md text-sm font-medium transition-all whitespace-nowrap ${
								isActive
									? "text-zinc-950 shadow-sm"
									: "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800"
							}`}
							style={isActive ? { backgroundColor: lang.color } : undefined}
						>
							{lang.label}
						</button>
					);
				})}
			</div>

			{/* Main split pane */}
			<div className="flex flex-1 overflow-hidden min-h-0">
				{/* Editor panel */}
				<div className="flex flex-col w-1/2 min-w-0 border-r border-zinc-800">
					{/* Editor header */}
					<div className="flex items-center justify-between px-4 py-2 bg-zinc-900 border-b border-zinc-800 shrink-0">
						<span className="text-xs text-zinc-500 font-medium uppercase tracking-wider">
							Editor
						</span>
						<span
							className="text-xs font-medium"
							style={{ color: activeLang.color }}
						>
							{activeLang.label}
						</span>
					</div>

					{/* Code editor */}
					<Suspense
						fallback={
							<div className="flex-1 m-4 animate-pulse">Loading editor...</div>
						}
					>
						<CodeEditor
							value={code}
							onChange={setCode}
							language={language}
							className="flex-1 overflow-auto resize-none bg-zinc-950 text-zinc-100 font-mono text-sm leading-relaxed p-4 outline-none placeholder-zinc-700 min-h-0"
						/>
					</Suspense>

					{/* Dependencies section */}
					<div className="shrink-0 border-t border-zinc-800 bg-zinc-900">
						<button
							type="button"
							onClick={() => setShowDeps((v) => !v)}
							className="flex items-center gap-2 w-full px-4 py-2.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
						>
							<PackagePlus size={13} />
							<span className="font-medium uppercase tracking-wider">
								Dependencies
							</span>
							{showDeps ? (
								<ChevronUp size={13} className="ml-auto" />
							) : (
								<ChevronDown size={13} className="ml-auto" />
							)}
						</button>
						{showDeps && (
							<div className="px-3 pb-3">
								<textarea
									value={dependencies}
									onChange={(e) => setDependencies(e.target.value)}
									spellCheck={false}
									rows={3}
									placeholder={DEPS_PLACEHOLDER[language] || "One per line"}
									className="w-full bg-zinc-950 text-zinc-200 font-mono text-xs leading-relaxed p-2.5 rounded border border-zinc-700 outline-none focus:border-zinc-500 resize-none placeholder-zinc-700 transition-colors"
								/>
							</div>
						)}
					</div>

					{/* Run button */}
					<div className="shrink-0 p-3 bg-zinc-900 border-t border-zinc-800">
						<button
							type="button"
							onClick={handleRun}
							className={`flex items-center justify-center gap-2 w-full py-2.5 rounded-lg font-semibold text-sm transition-all ${
								isRunning
									? "bg-zinc-700 text-zinc-400 hover:bg-zinc-600 cursor-pointer"
									: "text-zinc-950 shadow-lg active:scale-[0.98]"
							}`}
							style={
								!isRunning
									? {
											backgroundColor: activeLang.color,
											boxShadow: `0 0 20px ${activeLang.color}40`,
										}
									: undefined
							}
						>
							{isRunning ? (
								<>
									<Loader2 size={16} className="animate-spin" />
									Running… (click to cancel)
								</>
							) : (
								<>
									<Play size={16} fill="currentColor" />
									Run Code
								</>
							)}
						</button>
					</div>
				</div>

				{/* Output panel */}
				<div className="flex flex-col w-1/2 min-w-0">
					{/* Output header */}
					<div className="flex items-center justify-between px-4 py-2 bg-zinc-900 border-b border-zinc-800 shrink-0">
						<span className="text-xs text-zinc-500 font-medium uppercase tracking-wider">
							Output
						</span>
						{hasRun && (
							<button
								type="button"
								onClick={handleClear}
								className="flex items-center gap-1 text-xs text-zinc-600 hover:text-zinc-400 transition-colors"
							>
								<Trash2 size={12} />
								Clear
							</button>
						)}
					</div>

					{/* Output content */}
					<div
						ref={outputRef}
						className="flex-1 overflow-y-auto bg-zinc-950 p-4 font-mono text-sm min-h-0"
					>
						{!hasRun && (
							<div className="flex flex-col items-center justify-center h-full gap-3 text-zinc-700 select-none">
								<Play size={32} />
								<span className="text-sm">
									Press{" "}
									<span className="font-semibold text-zinc-500">Run Code</span>{" "}
									to see output
								</span>
							</div>
						)}

						{hasRun && outputLines.length === 0 && isRunning && (
							<div className="flex items-center gap-2 text-zinc-600">
								<Loader2 size={14} className="animate-spin" />
								<span>Starting…</span>
							</div>
						)}

						{outputLines.map((line) => (
							<div key={line.id} className="leading-relaxed">
								<ChunkLine chunk={line.chunk} />
							</div>
						))}

						{isRunning && outputLines.length > 0 && (
							<div className="flex items-center gap-1.5 mt-1 text-zinc-700">
								<span className="inline-block w-1.5 h-1.5 rounded-full bg-zinc-700 animate-pulse" />
								<span className="inline-block w-1.5 h-1.5 rounded-full bg-zinc-700 animate-pulse [animation-delay:0.2s]" />
								<span className="inline-block w-1.5 h-1.5 rounded-full bg-zinc-700 animate-pulse [animation-delay:0.4s]" />
							</div>
						)}
					</div>
				</div>
			</div>
		</div>
	);
}
