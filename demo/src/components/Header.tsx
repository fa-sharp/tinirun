import { Terminal } from "lucide-react";

export default function Header() {
	return (
		<header className="flex items-center gap-3 px-5 h-14 bg-zinc-900 border-b border-zinc-800 shrink-0">
			<div className="flex items-center gap-2.5">
				<div className="flex items-center justify-center w-7 h-7 rounded-md bg-emerald-500/10 text-emerald-400">
					<Terminal size={15} strokeWidth={2.5} />
				</div>
				<span className="text-white font-bold text-lg tracking-tight">
					tinirun
				</span>
			</div>
			<div className="h-4 w-px bg-zinc-700 mx-1" />
			<span className="text-zinc-500 text-sm">sandboxed code runner</span>
		</header>
	);
}
