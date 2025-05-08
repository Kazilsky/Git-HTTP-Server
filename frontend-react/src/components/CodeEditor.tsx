import React, { useState, useEffect } from 'react';
import Editor from '@monaco-editor/react';

/**
 * Props for the CodeEditor component
 * @interface CodeEditorProps
 * @property {string} value - The initial content of the editor
 * @property {string} language - The language for syntax highlighting (e.g., 'javascript', 'typescript', 'rust')
 * @property {boolean} readOnly - Whether the editor is read-only
 * @property {function} onChange - Callback function that is called when the content changes
 * @property {string} theme - The editor theme ('vs-dark' or 'light')
 * @property {number} height - The height of the editor in pixels
 */
interface CodeEditorProps {
  value: string;
  language: string;
  readOnly?: boolean;
  onChange?: (value: string | undefined) => void;
  theme?: 'vs-dark' | 'light';
  height?: number;
}

/**
 * CodeEditor component that provides a Monaco Editor instance
 * 
 * @component
 * @example
 * ```tsx
 * <CodeEditor 
 *   value="const hello = 'world';" 
 *   language="javascript" 
 *   onChange={(newValue) => console.log(newValue)} 
 * />
 * ```
 */
const CodeEditor: React.FC<CodeEditorProps> = ({
  value,
  language,
  readOnly = false,
  onChange,
  theme = 'vs-dark',
  height = 400
}) => {
  const [editorValue, setEditorValue] = useState<string>(value);

  useEffect(() => {
    setEditorValue(value);
  }, [value]);

  const handleEditorChange = (value: string | undefined) => {
    setEditorValue(value || '');
    if (onChange) {
      onChange(value);
    }
  };

  /**
   * Determines the appropriate language mode based on file extension
   * 
   * @param {string} filename - The name of the file
   * @returns {string} The language mode for Monaco Editor
   */
  const getLanguageFromFilename = (filename: string): string => {
    const extension = filename.split('.').pop()?.toLowerCase();
    
    const languageMap: Record<string, string> = {
      'js': 'javascript',
      'jsx': 'javascript',
      'ts': 'typescript',
      'tsx': 'typescript',
      'py': 'python',
      'rb': 'ruby',
      'java': 'java',
      'c': 'c',
      'cpp': 'cpp',
      'h': 'cpp',
      'cs': 'csharp',
      'go': 'go',
      'rs': 'rust',
      'php': 'php',
      'html': 'html',
      'css': 'css',
      'scss': 'scss',
      'json': 'json',
      'md': 'markdown',
      'yml': 'yaml',
      'yaml': 'yaml',
      'sh': 'shell',
      'bash': 'shell',
      'sql': 'sql',
      'xml': 'xml',
      'toml': 'toml',
      'kt': 'kotlin',
      'swift': 'swift',
      'dart': 'dart',
    };
    
    return extension && languageMap[extension] ? languageMap[extension] : 'plaintext';
  };

  // If language is a filename, extract the language from it
  const editorLanguage = language.includes('.') ? getLanguageFromFilename(language) : language;

  return (
    <div className="border rounded-md overflow-hidden">
      <Editor
        height={height}
        language={editorLanguage}
        value={editorValue}
        theme={theme}
        options={{
          readOnly,
          minimap: { enabled: true },
          scrollBeyondLastLine: false,
          fontSize: 14,
          wordWrap: 'on',
          automaticLayout: true,
          lineNumbers: 'on',
          scrollbar: {
            vertical: 'visible',
            horizontal: 'visible',
            verticalScrollbarSize: 12,
            horizontalScrollbarSize: 12,
          }
        }}
        onChange={handleEditorChange}
        className="w-full"
      />
    </div>
  );
};

export default CodeEditor;
