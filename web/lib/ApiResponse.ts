import { NextResponse } from "next/server";

export class ApiResponse {
  status: number;
  data: any;
  error: any;

  constructor() {
    this.status = 0;
    this.data = null;
    this.error = null;
  }

  success(data: any, statusCode: number = 200) {
    this.status = 1;
    this.data = data;
    this.error = null;
    return NextResponse.json(this, { status: statusCode });
  }

  failure(
    code: number,
    message: string,
    field: string | null = null,
    statusCode: number = 400
  ) {
    this.status = 0;
    this.data = null;
    this.error = {
      error_code: code,
      error_message: message,
      error_field: field,
    };
    return NextResponse.json(this, { status: statusCode });
  }
}
