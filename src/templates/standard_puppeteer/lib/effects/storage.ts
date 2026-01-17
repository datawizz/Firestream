// Storage Effects (S3 and Local)

import { BaseEffect, EffectError, ExecutionContext } from './index';
import { UploadS3Params, CompressionType } from '../types';
import { S3Client, PutObjectCommand, PutObjectCommandInput } from '@aws-sdk/client-s3';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as zlib from 'zlib';
import { promisify } from 'util';

const gzip = promisify(zlib.gzip);
const brotli = promisify(zlib.brotliCompress);

export class UploadS3Effect extends BaseEffect<UploadS3Params, string, { bucket: string; key: string; size: number }> {
  private s3Client: S3Client;
  
  constructor(
    id: string,
    type: string,
    params: UploadS3Params,
    dependencies: string[] = []
  ) {
    super(id, type, params, dependencies);
    
    const config: any = {
      region: process.env['AWS_REGION'] || 'us-east-1'
    };
    
    // Only add endpoint if it's defined
    if (process.env['S3_ENDPOINT']) {
      config.endpoint = process.env['S3_ENDPOINT'];
    }
    
    // Only add credentials if both keys are present
    if (process.env['AWS_ACCESS_KEY_ID'] && process.env['AWS_SECRET_ACCESS_KEY']) {
      config.credentials = {
        accessKeyId: process.env['AWS_ACCESS_KEY_ID'],
        secretAccessKey: process.env['AWS_SECRET_ACCESS_KEY']
      };
    }
    
    this.s3Client = new S3Client(config);
  }
  
  async execute(input: string, context: ExecutionContext): Promise<{ bucket: string; key: string; size: number }> {
    const key = this.generateKey(context);
    this.log('info', `Uploading to S3: s3://${this.params.bucket}/${key}`);
    
    try {
      // Prepare data for upload
      let data: Buffer = Buffer.from(input, 'utf-8');
      let contentEncoding: string | undefined;
      
      // Apply compression if requested
      if (this.params.compression && this.params.compression !== 'none') {
        const originalSize = data.length;
        data = await this.compress(data, this.params.compression);
        contentEncoding = this.params.compression;
        
        this.log('info', `Compressed data from ${originalSize} to ${data.length} bytes (${Math.round((1 - data.length / originalSize) * 100)}% reduction)`);
      }
      
      // Prepare S3 upload parameters
      const uploadParams: PutObjectCommandInput = {
        Bucket: this.params.bucket,
        Key: key,
        Body: data,
        ContentType: this.params.content_type || 'application/json',
        ContentEncoding: contentEncoding,
        Metadata: {
          'workflow-id': context.workflowId,
          'execution-id': context.executionId,
          'effect-id': this.id,
          'timestamp': new Date().toISOString()
        }
      };
      
      if (this.params.acl) {
        uploadParams.ACL = this.params.acl as any;
      }
      
      // Upload to S3
      const command = new PutObjectCommand(uploadParams);
      await this.s3Client.send(command);
      
      const result = {
        bucket: this.params.bucket,
        key,
        size: data.length
      };
      
      this.log('info', `Successfully uploaded to S3`, result);
      
      // Store upload details in context
      context.data.set(`${this.id}_s3_location`, `s3://${result.bucket}/${result.key}`);
      context.data.set(`${this.id}_s3_size`, result.size);
      
      return result;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to upload to S3: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error as Error
      );
    }
  }
  
  private generateKey(context: ExecutionContext): string {
    const now = new Date();
    const replacements: Record<string, string> = {
      '{{ workflow_id }}': context.workflowId,
      '{{ execution_id }}': context.executionId,
      '{{ date }}': now.toISOString().split('T')[0] || '',
      '{{ timestamp }}': now.getTime().toString(),
      '{{ year }}': now.getFullYear().toString(),
      '{{ month }}': (now.getMonth() + 1).toString().padStart(2, '0'),
      '{{ day }}': now.getDate().toString().padStart(2, '0'),
      '{{ hour }}': now.getHours().toString().padStart(2, '0'),
      '{{ minute }}': now.getMinutes().toString().padStart(2, '0')
    };
    
    let key = this.params.key_template;
    for (const [placeholder, value] of Object.entries(replacements)) {
      key = key.replace(new RegExp(placeholder.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'g'), value);
    }
    
    return key;
  }
  
  private async compress(data: Buffer, compression: CompressionType): Promise<Buffer> {
    switch (compression) {
      case 'gzip':
        return await gzip(data);
      case 'brotli':
        return await brotli(data);
      default:
        return data;
    }
  }
  
  override validate(): void {
    super.validate();
    
    if (!this.params.bucket) {
      throw new Error('S3 bucket is required');
    }
    
    if (!this.params.key_template) {
      throw new Error('S3 key template is required');
    }
  }
}

// Local file storage effect
export class SaveLocalEffect extends BaseEffect<{ filename: string; directory?: string }, string, string> {
  async execute(input: string, context: ExecutionContext): Promise<string> {
    const directory = this.params.directory || `output/${context.workflowId}`;
    const filepath = path.join(directory, this.params.filename);
    
    this.log('info', `Saving to local file: ${filepath}`);
    
    try {
      // Ensure directory exists
      await fs.mkdir(directory, { recursive: true });
      
      // Write file
      await fs.writeFile(filepath, input, 'utf-8');
      
      const stats = await fs.stat(filepath);
      this.log('info', `Successfully saved ${stats.size} bytes to ${filepath}`);
      
      // Store file path in context
      context.data.set(`${this.id}_filepath`, filepath);
      context.data.set(`${this.id}_size`, stats.size);
      
      return filepath;
    } catch (error) {
      throw new EffectError(
        this.id,
        this.type,
        `Failed to save file: ${error instanceof Error ? error.message : 'Unknown error'}`,
        error as Error
      );
    }
  }
  
  override validate(): void {
    super.validate();
    
    if (!this.params.filename) {
      throw new Error('Filename is required');
    }
  }
}