import CodeMirror from "@uiw/react-codemirror";

export const Editor = ({ content, onChange, extension }) => {
    return (
        <div className="flex flex-col gap-2 h-full">
            <CodeMirror
                value={content}
                height="100%"
                extensions={extension}
                onChange={onChange}
                theme="dark"
                className="text-sm md:text-base overflow-auto h-full"
                basicSetup={{
                    lineNumbers: true,
                    highlightActiveLine: true,
                    bracketMatching: true,
                    closeBrackets: true,
                    autocompletion: true,
                    foldGutter: true,
                }}
            />
        </div>
    );
};
