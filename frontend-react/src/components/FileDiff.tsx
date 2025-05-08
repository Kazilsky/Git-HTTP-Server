import React from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

/**
 * Interface for a line in a diff
 * @interface DiffLine
 * @property {string} content - The content of the line
 * @property {string} type - The type of change: 'added', 'removed', 'unchanged'
 * @property {number} oldLineNumber - The line number in the old file (null for added lines)
 * @property {number} newLineNumber - The line number in the new file (null for removed lines)
 */
interface DiffLine {
  content: string;
  type: 'added' | 'removed' | 'unchanged';
  oldLineNumber: number | null;
  newLineNumber: number | null;
}

/**
 * Props for the FileDiff component
 * @interface FileDiffProps
 * @property {string} filename - The name of the file being diffed
 * @property {DiffLine[]} diffLines - The lines of the diff
 * @property {boolean} expanded - Whether the diff is expanded or collapsed
 * @property {function} onToggleExpand - Callback function when the expand/collapse button is clicked
 */
interface FileDiffProps {
  filename: string;
  diffLines: DiffLine[];
  expanded?: boolean;
  onToggleExpand?: () => void;
}

/**
 * FileDiff component that displays a diff between two versions of a file
 * 
 * @component
 * @example
 * ```tsx
 * <FileDiff 
 *   filename="src/main.js"
 *   diffLines={[
 *     { content: "function hello() {", type: "unchanged", oldLineNumber: 1, newLineNumber: 1 },
 *     { content: "  console.log('Hello');", type: "removed", oldLineNumber: 2, newLineNumber: null },
 *     { content: "  console.log('Hello, World!');", type: "added", oldLineNumber: null, newLineNumber: 2 },
 *     { content: "}", type: "unchanged", oldLineNumber: 3, newLineNumber: 3 }
 *   ]}
 *   expanded={true}
 * />
 * ```
 */
const FileDiff: React.FC<FileDiffProps> = ({
  filename,
  diffLines,
  expanded = true,
  onToggleExpand = () => {}
}) => {
  /**
   * Determines the language for syntax highlighting based on file extension
   * 
   * @param {string} filename - The name of the file
   * @returns {string} The language for syntax highlighting
   */
  const getLanguageFromFilename = (filename: string): string => {
    const extension = filename.split('.').pop()?.toLowerCase();
    
    const languageMap: Record<string, string> = {
      'js': 'javascript',
      'jsx': 'jsx',
      'ts': 'typescript',
      'tsx': 'tsx',
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
      'sh': 'bash',
      'bash': 'bash',
      'sql': 'sql',
      'xml': 'xml',
      'toml': 'toml',
      'kt': 'kotlin',
      'swift': 'swift',
      'dart': 'dart',
    };
    
    return extension && languageMap[extension] ? languageMap[extension] : 'plaintext';
  };

  const language = getLanguageFromFilename(filename);

  /**
   * Renders a line number cell
   * 
   * @param {number|null} lineNumber - The line number to display
   * @returns {JSX.Element} The rendered line number cell
   */
  const renderLineNumber = (lineNumber: number | null): JSX.Element => {
    return (
      <td className="text-right pr-2 w-12 select-none text-gray-500 text-xs border-r">
        {lineNumber !== null ? lineNumber : ' '}
      </td>
    );
  };

  /**
   * Renders a line of code with appropriate styling based on its type
   * 
   * @param {DiffLine} line - The line to render
   * @param {number} index - The index of the line in the diffLines array
   * @returns {JSX.Element} The rendered line
   */
  const renderLine = (line: DiffLine, index: number): JSX.Element => {
    const bgColor = line.type === 'added' 
      ? 'bg-green-50' 
      : line.type === 'removed' 
        ? 'bg-red-50' 
        : '';
    
    const textColor = line.type === 'added' 
      ? 'text-green-800' 
      : line.type === 'removed' 
        ? 'text-red-800' 
        : 'text-gray-800';
    
    const prefix = line.type === 'added' 
      ? '+ ' 
      : line.type === 'removed' 
        ? '- ' 
        : '  ';
    
    return (
      <tr key={index} className={`${bgColor} ${textColor}`}>
        {renderLineNumber(line.oldLineNumber)}
        {renderLineNumber(line.newLineNumber)}
        <td className="pl-2 font-mono whitespace-pre">
          <SyntaxHighlighter 
            language={language}
            style={vscDarkPlus}
            customStyle={{
              margin: 0,
              padding: 0,
              background: 'transparent',
              fontSize: '0.85rem',
              lineHeight: '1.5',
            }}
            codeTagProps={{
              style: {
                fontSize: 'inherit',
                lineHeight: 'inherit',
              }
            }}
          >
            {prefix + line.content}
          </SyntaxHighlighter>
        </td>
      </tr>
    );
  };

  /**
   * Calculates statistics about the diff
   * 
   * @returns {Object} Object containing the number of added, removed, and changed lines
   */
  const getDiffStats = () => {
    const added = diffLines.filter(line => line.type === 'added').length;
    const removed = diffLines.filter(line => line.type === 'removed').length;
    const changed = added + removed;
    
    return { added, removed, changed };
  };

  const stats = getDiffStats();

  return (
    <div className="border rounded-md overflow-hidden mb-4">
      <div className="bg-gray-100 px-4 py-2 flex justify-between items-center">
        <div className="font-mono text-sm flex items-center">
          <span className="mr-4">{filename}</span>
          <span className="text-xs bg-gray-200 rounded-full px-2 py-1 flex items-center">
            <span className="text-green-600 mr-2">+{stats.added}</span>
            <span className="text-red-600">-{stats.removed}</span>
          </span>
        </div>
        <button 
          onClick={onToggleExpand}
          className="text-gray-600 hover:text-gray-800"
        >
          {expanded ? (
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 15l7-7 7 7" />
            </svg>
          ) : (
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          )}
        </button>
      </div>
      
      {expanded && (
        <div className="overflow-x-auto">
          <table className="w-full border-collapse">
            <tbody>
              {diffLines.map((line, index) => renderLine(line, index))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};

export default FileDiff;
