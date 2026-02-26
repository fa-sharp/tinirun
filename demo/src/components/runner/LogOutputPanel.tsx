import type { TinirunSchemas } from "@tinirun/client";
import { Loader2, Play, Trash2 } from "lucide-react";
import { useEffect, useRef } from "react";
import { LogChunk } from "./LogChunk";

type Chunk = TinirunSchemas["CodeRunnerChunk"];

export interface LogOutputLine {
	id: number;
	chunk: Chunk;
}

export function LogOutputPanel({
	status,
	outputLines,
	handleClear,
}: {
	status: "initial" | "running" | "completed";
	outputLines: LogOutputLine[];
	handleClear: () => void;
}) {
	const outputRef = useRef<HTMLDivElement>(null);
	useEffect(() => {
		const el = outputRef.current;
		if (el && outputLines.length > 0) {
			el.scrollTop = el.scrollHeight;
		}
	}, [outputLines.length]);

	return (
		<div className="flex flex-col w-1/2 min-w-0">
			{/* Output header */}
			<div className="flex items-center justify-between px-4 py-2 bg-zinc-900 border-b border-zinc-800 shrink-0">
				<span className="text-xs text-zinc-500 font-medium uppercase tracking-wider">
					Output
				</span>
				<button
					type="button"
					onClick={handleClear}
					className="flex items-center gap-1 text-xs text-zinc-600 hover:text-zinc-400 transition-colors"
				>
					<Trash2 size={12} />
					Clear
				</button>
			</div>

			{/* Output content */}
			<div
				ref={outputRef}
				className="flex-1 overflow-y-auto bg-zinc-950 p-4 font-mono text-sm min-h-0"
			>
				{status === "initial" && (
					<div className="flex flex-col items-center justify-center h-full gap-3 text-zinc-700 select-none">
						<Play size={32} />
						<span className="text-sm">
							Press{" "}
							<span className="font-semibold text-zinc-500">Run Code</span> to
							see output
						</span>
					</div>
				)}

				{status === "running" && outputLines.length === 0 && (
					<div className="flex items-center gap-2 text-zinc-600">
						<Loader2 size={14} className="animate-spin" />
						<span>Starting…</span>
					</div>
				)}

				{outputLines.map((line) => (
					<div key={line.id} className="leading-relaxed">
						<LogChunk chunk={line.chunk} />
					</div>
				))}

				{status === "running" && outputLines.length > 0 && (
					<div className="flex items-center gap-1.5 mt-1 text-zinc-700">
						<span className="inline-block w-1.5 h-1.5 rounded-full bg-zinc-700 animate-pulse" />
						<span className="inline-block w-1.5 h-1.5 rounded-full bg-zinc-700 animate-pulse [animation-delay:0.2s]" />
						<span className="inline-block w-1.5 h-1.5 rounded-full bg-zinc-700 animate-pulse [animation-delay:0.4s]" />
					</div>
				)}
			</div>
		</div>
	);
}
