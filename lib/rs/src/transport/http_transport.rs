/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements. See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership. The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License. You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::io::{self, Cursor, BufStream};
use std::io::prelude::*;

use hyper::header::{Headers, Host, ContentType, ContentLength, UserAgent};
use hyper::http::parse_response;
use httparse;

use super::Transport;

pub struct ClientHTTPTransport<T: Transport> {
    host: String,
    path: String,
    transport: BufStream<T>,
    write_buf: Vec<u8>,
    read_buf: Vec<u8>,
    read_offset: usize,
    read_headers: bool
}

impl<T: Transport> ClientHTTPTransport<T> {
    // TODO: Add alternate constructor
    pub fn new(transport: T, host: String, path: String) -> Self {
        ClientHTTPTransport { transport: BufStream::new(transport), host: host, path: path,
                              write_buf: Vec::new(), read_buf: Vec::new(), read_headers: true,
                              read_offset: 0 }
    }

    fn get_headers(&self) -> String {
        let mut h = Headers::new();
        h.set(Host { hostname: self.host.clone(), port: None });
        h.set(ContentType("application/x-thrift".parse().unwrap()));
        h.set(ContentLength(self.write_buf.len() as u64));
        h.set(UserAgent("Rust/ClientHTTPTransport".to_owned()));
        h.to_string()
    }

    // fn read_headers(&self) -> u64 {
    //     let buf = [0u8; 200];
    //     let mut buf_ref = &buf[..];
    //     let mut reader = ::hyper::buffer::BufReader::new(&mut buf_ref);
    //     let mut response = parse_response(&mut reader);
    //     println!("{:?}", response);
    //     println!("{:?}", self.read_buf);
    //     let mut response = response.unwrap();
    //     let &ContentLength(len) = response.headers.get::<ContentLength>().unwrap();
    //     len
    // }

    fn read_headers(&mut self) -> u64 {
        let mut buf = Vec::new();
        loop {
            let mut headers = [httparse::EMPTY_HEADER; 10];
            let mut resp = httparse::Response::new(&mut headers);
            self.transport.read_until('\n' as u8, &mut buf);
            println!("{:?}", ::std::str::from_utf8(&*buf));
            let res = resp.parse(&buf).unwrap();

            if res.is_complete() {
                println!("Complete");
                self.read_headers = false;
                let mut len = None;
                for h in resp.headers.iter() {
                    println!("{}: {}", h.name, ::std::str::from_utf8(h.value).unwrap());
                    if h.name == "content-length" { // TODO: Case
                        len = Some(::std::str::from_utf8(h.value).unwrap().parse().unwrap());
                        break;
                    }
                }
                return len.expect("No Content-Length");
            }
        }
    }
}

impl<T: Transport> Read for ClientHTTPTransport<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_buf.len() == self.read_offset {
            self.read_buf.clear();
            self.read_headers = true;
            self.read_offset = 0;
        }
        if self.read_headers {
            let len = self.read_headers();
            let transport_ref = &mut self.transport;
            transport_ref.take(len).read_to_end(&mut self.read_buf);
            println!("read_buf: {:?}", self.read_buf);
        }
        let res = (&self.read_buf[self.read_offset..]).read(buf);
        self.read_offset += buf.len();
        println!("buf: {:?}", buf);
        res
    }
}

impl<T: Transport> Write for ClientHTTPTransport<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let headers = self.get_headers();

        self.transport.write(format!("POST {} HTTP/1.1\r\n", self.path).as_bytes()).unwrap();
        self.transport.write(headers.to_string().as_bytes()).unwrap();
        self.transport.write("\r\n".as_bytes()).unwrap();
        self.transport.write(&self.write_buf).unwrap();
        self.write_buf.clear();
        self.transport.flush()
    }
}

impl<T: Transport> Transport for ClientHTTPTransport<T> { }
