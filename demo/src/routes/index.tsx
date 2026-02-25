import { createFileRoute } from "@tanstack/react-router";
import { CodeRunner } from "@/components/CodeRunner";

export const Route = createFileRoute("/")({
	component: HomePage,
	ssr: "data-only",
});

function HomePage() {
	return (
		<div className="h-full flex flex-col overflow-hidden">
			<CodeRunner />
		</div>
	);
}
