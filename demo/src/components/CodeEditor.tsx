import { indentUnit, StreamLanguage } from "@codemirror/language";
import type { TinirunSchemas } from "@tinirun/client";
import CodeMirror, { EditorState, type Extension } from "@uiw/react-codemirror";
import { useEffect, useState } from "react";

const DEFAULT_EXTENSIONS = [
	EditorState.tabSize.of(4),
	indentUnit.of(" ".repeat(4)),
];

interface CodeEditorProps {
	value: string;
	language: TinirunSchemas["CodeRunnerLanguage"];
	className?: string;
	onChange?: (value: string) => void;
}

export default function CodeEditor({
	value,
	className,
	language,
	onChange,
}: CodeEditorProps) {
	const [extensions, setExtensions] = useState<Extension[]>(DEFAULT_EXTENSIONS);

	useEffect(() => {
		async function loadExtensions() {
			const languageExtension = await getExtension(language);
			setExtensions([...DEFAULT_EXTENSIONS, languageExtension]);
		}
		loadExtensions();
	}, [language]);

	return (
		<CodeMirror
			value={value}
			onChange={onChange}
			className={className}
			theme="dark"
			extensions={extensions}
		/>
	);
}

async function getExtension(language: TinirunSchemas["CodeRunnerLanguage"]) {
	switch (language) {
		case "javascript":
		case "typescript":
			return await import("@codemirror/lang-javascript").then((mod) =>
				mod.javascript({ typescript: true }),
			);
		case "python":
			return await import("@codemirror/lang-python").then((mod) =>
				mod.python(),
			);
		case "go":
			return await import("@codemirror/lang-go").then((mod) => mod.go());
		case "rust":
			return await import("@codemirror/lang-rust").then((mod) => mod.rust());
		case "bash": {
			const { shell } = await import("@codemirror/legacy-modes/mode/shell");
			return StreamLanguage.define(shell);
		}
	}
}
