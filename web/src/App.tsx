import { useState } from 'react';
import { Editor } from './components/Editor';
import { Navbar } from './components/Navbar';
import { languages } from './consts';
import { type Extension } from '@codemirror/state';

interface Language {
  value: string;
  name: string;
  defaultCode: string;
  extension?: Extension;
}

interface LanguageContents {
  [key: string]: string;
}

function App() {
  const [selectedLanguage, setSelectedLanguage] = useState<string>('python');
  const [languageContents, setLanguageContents] = useState<LanguageContents>(() => {
    const initialContents: LanguageContents = {};
    languages.forEach((lang: Language) => {
      initialContents[lang.value] = lang.defaultCode;
    });
    return initialContents;
  });

  const currentLanguage = languages.find((lang: Language) => lang.value === selectedLanguage);
  const currentContent: string = languageContents[selectedLanguage] || '';

  const handleLanguageChange = (newLanguage: string): void => {
    setSelectedLanguage(newLanguage);
  };

  const handleContentChange = (newContent: string): void => {
    setLanguageContents((prev: LanguageContents) => ({
      ...prev,
      [selectedLanguage]: newContent,
    }));
  };

  return (
    <div className="flex flex-col h-screen">
      <Navbar />
      <div className="flex-shrink-0 px-5 py-2.5 bg-gray-100 border-b border-gray-300 flex items-center gap-2.5">
        <label htmlFor="language-select" className="font-bold">
          Language:
        </label>
        <select
          id="language-select"
          value={selectedLanguage}
          onChange={(e: React.ChangeEvent<HTMLSelectElement>) => handleLanguageChange(e.target.value)}
          className="px-2.5 py-1 border border-gray-300 rounded text-sm"
        >
          {languages.map((lang: Language) => (
            <option key={lang.value} value={lang.value}>
              {lang.name}
            </option>
          ))}
        </select>
      </div>
      <div className="flex-grow h-[90vh]">
        <Editor
          content={currentContent}
          onChange={handleContentChange}
          extension={currentLanguage?.extension ? [currentLanguage.extension] : []}
        />
      </div>
    </div>
  );
}

export default App;
