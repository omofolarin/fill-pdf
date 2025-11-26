#!/usr/bin/env bun
/**
 * PDF merging and flattening using pdf-lib
 * Alternative to PyPDF2 implementation
 */

import { PDFDocument } from 'pdf-lib';
import { readFileSync, writeFileSync } from 'fs';

interface MergeOptions {
  template: string;
  overlay: string;
  output: string;
  flatten: boolean;
}

async function mergePdfs(options: MergeOptions): Promise<void> {
  const startTime = performance.now();
  
  try {
    // Load PDFs
    const templateBytes = readFileSync(options.template);
    const overlayBytes = readFileSync(options.overlay);
    
    const template = await PDFDocument.load(templateBytes);
    const overlay = await PDFDocument.load(overlayBytes);
    
    // Merge pages
    const overlayPages = overlay.getPages();
    const templatePages = template.getPages();
    
    for (let i = 0; i < templatePages.length; i++) {
      if (i < overlayPages.length) {
        const [embeddedPage] = await template.embedPages([overlayPages[i]]);
        const { width, height } = templatePages[i].getSize();
        
        // Draw overlay on template page
        templatePages[i].drawPage(embeddedPage, {
          x: 0,
          y: 0,
          width,
          height,
        });
      }
    }
    
    // Flatten form if requested
    if (options.flatten) {
      try {
        const form = template.getForm();
        form.flatten();
      } catch (e) {
        // No form to flatten or already flat
      }
    }
    
    // Save merged PDF
    const mergedBytes = await template.save();
    writeFileSync(options.output, mergedBytes);
    
    const endTime = performance.now();
    const duration = (endTime - startTime).toFixed(2);
    
    console.log(`SUCCESS:${duration}ms`);
  } catch (error) {
    console.error(`ERROR: ${error instanceof Error ? error.message : String(error)}`);
    process.exit(1);
  }
}

// Parse command line arguments
const args = process.argv.slice(2);
const options: MergeOptions = {
  template: '',
  overlay: '',
  output: '',
  flatten: false,
};

for (let i = 0; i < args.length; i++) {
  switch (args[i]) {
    case '--template':
      options.template = args[++i];
      break;
    case '--overlay':
      options.overlay = args[++i];
      break;
    case '--output':
      options.output = args[++i];
      break;
    case '--flatten':
      options.flatten = true;
      break;
  }
}

if (!options.template || !options.overlay || !options.output) {
  console.error('Usage: bun merge_pdfs.ts --template <file> --overlay <file> --output <file> [--flatten]');
  process.exit(1);
}

mergePdfs(options);
