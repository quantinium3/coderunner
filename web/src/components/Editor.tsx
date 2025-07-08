import CodeMirror from "@uiw/react-codemirror";

export const Editor = ({ content, onChange, extension }) => {
    return (
        <div className="h-full overflow-hidden">
            <CodeMirror
                value={content}
                height="100%"
                extensions={extension}
                onChange={onChange}
                theme="dark"
                className="text-sm md:text-base h-full"
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
