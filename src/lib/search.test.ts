import { describe, it, expect } from 'vitest';
import { parseSearchTokens, fieldsMatchAllTokens, matchRanges, highlightSegments } from './search';

describe('parseSearchTokens', () => {
  it('splits on whitespace, lowercases, drops empties and duplicates', () => {
    expect(parseSearchTokens('  Abc   GHI abc ')).toEqual(['abc', 'ghi']);
    expect(parseSearchTokens('   ')).toEqual([]);
  });
});

describe('fieldsMatchAllTokens', () => {
  it('matches tokens as case-insensitive substrings', () => {
    expect(fieldsMatchAllTokens(['abc def ghi'], ['abc', 'ghi'])).toBe(true);
    expect(fieldsMatchAllTokens(['ABCDEFGHI'], ['abc', 'ghi'])).toBe(true);
  });

  it('requires every token (AND), each in any field', () => {
    const fields = ['Title abc', 'open def', 'closed ghi'];
    expect(fieldsMatchAllTokens(fields, ['abc', 'def', 'ghi'])).toBe(true);
    expect(fieldsMatchAllTokens(fields, ['abc', 'xyz'])).toBe(false);
  });

  it('never matches a token across two fields', () => {
    expect(fieldsMatchAllTokens(['ab', 'cd'], ['bc'])).toBe(false);
  });

  it('treats regex characters literally', () => {
    expect(fieldsMatchAllTokens(['cost (eur)'], ['(eur)'])).toBe(true);
    expect(fieldsMatchAllTokens(['abc'], ['a.c'])).toBe(false);
  });
});

describe('matchRanges / highlightSegments', () => {
  it('finds every occurrence, case-insensitively', () => {
    expect(matchRanges('Abc abc', ['abc'])).toEqual([[0, 3], [4, 7]]);
  });

  it('merges overlapping tokens into one range', () => {
    expect(matchRanges('xabcdx', ['abc', 'bcd'])).toEqual([[1, 5]]);
  });

  it('splits text into hit / non-hit segments, keeping original case', () => {
    expect(highlightSegments('Hello World', ['world'])).toEqual([
      { text: 'Hello ', hit: false },
      { text: 'World', hit: true },
    ]);
    expect(highlightSegments('Hello', [])).toEqual([{ text: 'Hello', hit: false }]);
  });
});
