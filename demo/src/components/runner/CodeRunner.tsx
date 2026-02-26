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
} from "lucide-react";
import React, { Suspense, useCallback, useRef, useState } from "react";
import { runCodeSnippetServerFn } from "@/api/runCode";
import { DEFAULT_CODE, DEPS_PLACEHOLDER, LANGUAGES } from "./constants";
import { type LogOutputLine, LogOutputPanel } from "./LogOutputPanel";

const CodeEditor = React.lazy(() => import("./CodeEditor"));

type Language = TinirunSchemas["CodeRunnerLanguage"];

let nextId = 0;

export function CodeRunner() {
	const [language, setLanguage] = useState<Language>("python");
	const [code, setCode] = useState(DEFAULT_CODE.python);
	const [dependencies, setDependencies] = useState("");
	const [showDeps, setShowDeps] = useState(false);
	const [status, setStatus] = useState<"initial" | "running" | "completed">(
		"initial",
	);
	const [outputLines, setOutputLines] = useState<LogOutputLine[]>([]);
	const readerRef =
		useRef<ReadableStreamDefaultReader<EventSourceMessage> | null>(null);

	const handleLanguageChange = useCallback((lang: Language) => {
		setLanguage(lang);
		setCode(DEFAULT_CODE[lang]);
		setDependencies("");
	}, []);

	const handleRun = async () => {
		if (status === "running") {
			await readerRef.current?.cancel();
			readerRef.current = null;
			setStatus("initial");
			return;
		}

		const deps = dependencies
			.split("\n")
			.map((d) => d.trim())
			.filter(Boolean);

		setStatus("running");
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
			if (!res.ok) throw new Error(await res.text());
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
			setStatus("completed");
		}
	};

	const handleClear = () => {
		setOutputLines([]);
		setStatus("initial");
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
							<div className="flex-1 m-4 text-zinc-400 animate-pulse">
								Loading editor...
							</div>
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
							className={`flex items-center justify-center gap-2 w-full py-2.5 rounded-lg cursor-pointer font-semibold text-sm transition-all ${
								status === "running"
									? "bg-zinc-700 text-zinc-400 hover:bg-zinc-600"
									: "text-zinc-950 shadow-lg active:scale-[0.98]"
							}`}
							style={
								status !== "running"
									? {
											backgroundColor: activeLang.color,
											boxShadow: `0 0 20px ${activeLang.color}40`,
										}
									: undefined
							}
						>
							{status === "running" ? (
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

				<LogOutputPanel
					status={status}
					outputLines={outputLines}
					handleClear={handleClear}
				/>
			</div>
		</div>
	);
}
