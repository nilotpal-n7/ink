// app/lib/ast-parser.ts

export function parseCodeIntoSemanticChunks(code: string, filePath: string) {
  const lines = code.split('\n');
  const chunks = [];
  let currentChunk: string[] = [];
  let startLine = 1;
  let blockName = "global_scope";

  // A fast semantic boundary detector
  const boundaryRegex = /(?:export\s+)?(?:async\s+)?(?:function|class|const\s+\w+\s*=\s*(?:async\s*)?\([^)]*\)\s*=>|impl\s+\w+)/;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    
    // If we hit a new function/class boundary
    if (boundaryRegex.test(line)) {
      // Save the previous block if it has content
      if (currentChunk.join('').trim().length > 0) {
        chunks.push({
          code: currentChunk.join('\n'),
          lines: `${startLine}-${i}`,
          filePath,
          blockName
        });
      }
      // Start a new block
      currentChunk = [];
      startLine = i + 1;
      
      // Try to extract the name for metadata
      const nameMatch = line.match(/(?:function|class|const|impl)\s+([a-zA-Z0-9_]+)/);
      blockName = nameMatch ? nameMatch[1] : `anonymous_block_L${startLine}`;
    }
    
    currentChunk.push(line);
  }
  
  // Push the final chunk
  if (currentChunk.join('').trim().length > 0) {
    chunks.push({
      code: currentChunk.join('\n'),
      lines: `${startLine}-${lines.length}`,
      filePath,
      blockName
    });
  }
  
  return chunks;
}
