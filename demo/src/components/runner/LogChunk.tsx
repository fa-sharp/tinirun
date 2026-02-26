import type { TinirunSchemas } from "@tinirun/client";

type Chunk = TinirunSchemas["CodeRunnerChunk"];

export function LogChunk({ chunk }: { chunk: Chunk }) {
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
