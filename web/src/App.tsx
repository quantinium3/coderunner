import { useEffect, useRef, useState } from 'react';
import { Editor } from './components/Editor';
import { type Extension } from '@codemirror/state';
import { python } from '@codemirror/lang-python';
import { loadLanguage } from '@uiw/codemirror-extensions-langs';
import axios from "axios";

type state = {
    value: string
    language: string;
    content: string;
    extension: Extension | unknown,
}

const States: state[] = [
    {
        value: 'c',
        language: 'c',
        content: '#include <stdio.h>\n\nint main() {\n    printf("Hello, World!\\n");\n    return 0;\n}',
        extension: loadLanguage('c')
    },
    {
        value: 'cpp',
        language: 'cpp',
        content: '#include <iostream>\nusing namespace std;\n\nint main() {\n    cout << "Hello, World!" << endl;\n    return 0;\n}',
        extension: loadLanguage('cpp')
    },
    {
        value: 'python',
        language: 'python',
        content: 'print("Hello, World!")',
        extension: loadLanguage('python')
    },
    {
        value: 'java',
        language: 'java',
        content: 'public class Main {\n    public static void main(String[] args) {\n        System.out.println("Hello, World!");\n    }\n}',
        extension: loadLanguage('java')
    },
    {
        value: 'javascript',
        language: 'javascript',
        content: 'console.log("Hello, World!");',
        extension: loadLanguage('javascript')
    }
];

function App() {
    const [editorState, setEditorState] = useState<state>({
        value: "python",
        content: `print("hello world")`,
        language: "python",
        extension: python(),
    })
    const [result, setResult] = useState("");
    const [stdin, setStdin] = useState("");

    const [leftWidth, setLeftWidth] = useState(50);
    const [topHeight, setTopHeight] = useState(50);
    const [isResizingHorizontal, setIsResizingHorizontal] = useState(false);
    const [isResizingVertical, setIsResizingVertical] = useState(false);

    const [isMobile, setIsMobile] = useState(false);
    const [showEditor, setShowEditor] = useState(true);

    const containerRef = useRef<HTMLDivElement>(null);
    const rightPanelRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const checkMobile = () => {
            setIsMobile(window.innerWidth < 768);
        };

        checkMobile();
        window.addEventListener('resize', checkMobile);

        return () => window.removeEventListener('resize', checkMobile);
    }, []);

    const onEditorStateChange = (content: string) => {
        setEditorState((prev: state) => ({
            ...prev,
            content: content
        }))
    }

    const onLanguageChange = (lang: string) => {
        const selectedState = States.find(state => state.value === lang);

        if (selectedState) {
            setEditorState((prev: state) => ({
                ...prev,
                value: selectedState.value,
                language: selectedState.language,
                content: selectedState.content,
                extension: selectedState.extension
            }))
        }
    }

    const compileCode = () => {
        axios.post("https:/comphub.quantinium.dev/api/v1/compile", {
            lang: editorState.language,
            content: editorState.content,
            stdin: stdin
        }).then((res) => {
            setResult(res.data.result);
            if (isMobile) {
                setShowEditor(false);
            }
        }).catch((err) => {
            setResult("Error: " + err.message);
            if (isMobile) {
                setShowEditor(false);
            }
        })
    }

    const onResultChange = (content: string) => {
        setResult(content);
    }

    const onStdinChange = (content: string) => {
        setStdin(content);
    }

    const handleHorizontalMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        setIsResizingHorizontal(true);
    };

    const handleVerticalMouseDown = (e: React.MouseEvent) => {
        e.preventDefault();
        setIsResizingVertical(true);
    };

    useEffect(() => {
        const handleMouseMove = (e: MouseEvent) => {
            if (isResizingHorizontal && containerRef.current) {
                const containerRect = containerRef.current.getBoundingClientRect();
                const newWidth = ((e.clientX - containerRect.left) / containerRect.width) * 100;
                setLeftWidth(Math.max(20, Math.min(80, newWidth)));
            }

            if (isResizingVertical && rightPanelRef.current) {
                const rightPanelRect = rightPanelRef.current.getBoundingClientRect();
                const newHeight = ((e.clientY - rightPanelRect.top) / rightPanelRect.height) * 100;
                setTopHeight(Math.max(10, Math.min(90, newHeight)));
            }
        };

        const handleMouseUp = () => {
            setIsResizingHorizontal(false);
            setIsResizingVertical(false);
        };

        if (isResizingHorizontal || isResizingVertical) {
            document.addEventListener('mousemove', handleMouseMove);
            document.addEventListener('mouseup', handleMouseUp);
        }

        return () => {
            document.removeEventListener('mousemove', handleMouseMove);
            document.removeEventListener('mouseup', handleMouseUp);
        };
    }, [isResizingHorizontal, isResizingVertical]);

    return (
        <div className="flex flex-col h-screen">
            <div className="flex-shrink-0 px-5 py-2.5 bg-[#222222] text-white flex items-center gap-2.5 border-b-1 border-zinc-700">
                <div className='font-bold'>Comphub</div>
                <div className="flex items-center gap-2 bg-zinc-800 text-white border-zinc-700">
                    <select
                        id="language-select"
                        value={editorState.value}
                        onChange={(e: React.ChangeEvent<HTMLSelectElement>) => onLanguageChange(e.target.value)}
                        className="border border-zinc-700 rounded text-sm bg-zinc-800 text-white px-2 py-[2px] hover:bg-zinc-800 focus:outline-none focus:ring-0"
                    >
                        {States.map((State: state) => (
                            <option key={State.value} value={State.value} className="bg-zinc-800">
                                <span>{State.language}</span>
                            </option>
                        ))}
                    </select>
                </div>

                <div>
                    <button
                        onClick={compileCode}
                        className="bg-zinc-900 border-zinc-700 border text-white rounded hover:bg-zinc-800 transition-colors px-2"
                    >
                        Compile
                    </button>
                </div>


            </div>

            <div className='flex flex-1 overflow-hidden' ref={containerRef}>
                {!isMobile && (
                    <>
                        <div
                            className="bg-zinc-900 border-r border-gray-300 flex flex-col overflow-auto"
                            style={{ width: `${leftWidth}%` }}
                        >
                            <div className="bg-[#222222] px-3 py-2 border-b-1 border-zinc-700 text-sm font-medium text-gray-100">
                                Code Editor
                            </div>
                            <div className="flex-1">
                                <Editor
                                    content={editorState.content}
                                    onChange={onEditorStateChange}
                                    extension={editorState.extension}
                                />
                            </div>
                        </div>

                        <div
                            className="w-[1px] bg-zinc-900 border-zinc-800 cursor-col-resize hover:bg-gray-700 transition-colors flex-shrink-0"
                            onMouseDown={handleHorizontalMouseDown}
                        />

                        <div
                            className="flex flex-col flex-1 overflow-auto"
                            style={{ width: `${100 - leftWidth}%` }}
                            ref={rightPanelRef}
                        >
                            <div
                                className="bg-white border-b border-gray-300 flex flex-col"
                                style={{ height: `${topHeight}%` }}
                            >
                                <div className="bg-[#222222] text-gray-100 px-2 py-[8px] border-b border-zinc-700 text-sm font-medium">
                                    Input (stdin)
                                </div>
                                <div className="flex-1 overflow-auto">
                                    <Editor
                                        content={stdin}
                                        onChange={onStdinChange}
                                        extension={[loadLanguage('shell')]}
                                    />
                                </div>
                            </div>

                            <div
                                className="h-[1px] bg-gray-700 cursor-row-resize hover:bg-gray-500 transition-colors flex-shrink-0"
                                onMouseDown={handleVerticalMouseDown}
                            />

                            <div
                                className="bg-white flex flex-col"
                                style={{ height: `${100 - topHeight}%` }}
                            >
                                <div className="bg-[#222222] px-3 py-[8px] border-b border-gray-700 text-sm font-medium text-gray-100">
                                    Output
                                </div>
                                <div className="flex-1 overflow-auto">
                                    <Editor
                                        content={result}
                                        onChange={onResultChange}
                                        extension={[loadLanguage('shell')]}
                                    />
                                </div>
                            </div>
                        </div>
                    </>
                )}

                {isMobile && (
                    <div className="flex-1 flex flex-col overflow-hidden">
                        {showEditor ? (
                            <div className="bg-zinc-900 flex flex-col overflow-auto flex-1">
                                <div className="bg-[#222222] px-3 py-2 border-b-1 border-zinc-700 text-sm font-medium text-gray-100 flex items-center justify-between">
                                    <span>Code Editor</span>
                                    <div className="flex gap-1">
                                        <button
                                            onClick={() => setShowEditor(true)}
                                            className={`px-2 py-1 text-xs rounded transition-colors ${showEditor
                                                    ? 'bg-orange-600 text-white'
                                                    : 'bg-zinc-700 text-gray-300 hover:bg-zinc-600'
                                                }`}
                                        >
                                            Editor
                                        </button>
                                        <button
                                            onClick={() => setShowEditor(false)}
                                            className={`px-2 py-1 text-xs rounded transition-colors ${!showEditor
                                                    ? 'bg-orange-600 text-white'
                                                    : 'bg-zinc-700 text-gray-300 hover:bg-zinc-600'
                                                }`}
                                        >
                                            I/O
                                        </button>
                                    </div>
                                </div>
                                <div className="flex-1">
                                    <Editor
                                        content={editorState.content}
                                        onChange={onEditorStateChange}
                                        extension={editorState.extension}
                                    />
                                </div>
                            </div>
                        ) : (
                            <div className="flex flex-col flex-1 overflow-hidden">
                                <div className="bg-white border-b border-gray-300 flex flex-col flex-1">
                                    <div className="bg-[#222222] text-gray-100 px-2 py-[8px] border-b border-zinc-700 text-sm font-medium flex items-center justify-between">
                                        <span>Input (stdin)</span>
                                        <div className="flex gap-1">
                                            <button
                                                onClick={() => setShowEditor(true)}
                                                className={`px-2 py-1 text-xs rounded transition-colors ${showEditor
                                                        ? 'bg-blue-600 text-white'
                                                        : 'bg-zinc-700 text-gray-300 hover:bg-zinc-600'
                                                    }`}
                                            >
                                                Editor
                                            </button>
                                            <button
                                                onClick={() => setShowEditor(false)}
                                                className={`px-2 py-1 text-xs rounded transition-colors ${!showEditor
                                                        ? 'bg-blue-600 text-white'
                                                        : 'bg-zinc-700 text-gray-300 hover:bg-zinc-600'
                                                    }`}
                                            >
                                                I/O
                                            </button>
                                        </div>
                                    </div>
                                    <div className="flex-1 overflow-auto">
                                        <Editor
                                            content={stdin}
                                            onChange={onStdinChange}
                                            extension={[loadLanguage('shell')]}
                                        />
                                    </div>
                                </div>

                                <div className="h-[1px] bg-gray-700 flex-shrink-0" />

                                <div className="bg-white flex flex-col flex-1">
                                    <div className="bg-[#222222] px-3 py-[8px] border-b border-gray-700 text-sm font-medium text-gray-100">
                                        Output
                                    </div>
                                    <div className="flex-1 overflow-auto">
                                        <Editor
                                            content={result}
                                            onChange={onResultChange}
                                            extension={[loadLanguage('shell')]}
                                        />
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}

export default App;
