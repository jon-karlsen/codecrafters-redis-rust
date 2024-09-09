use std::{sync::{mpsc, Arc, Mutex}, thread};


struct Worker {
    id    : usize,
    thread: Option<thread::JoinHandle<()>>,
}


impl Worker {
    fn new( id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>> ) -> Worker {
        let thread = thread::spawn( move || loop {
            let msg = receiver.lock().unwrap().recv();

            match msg {
                Err( _ )   => {
                    println!( "worker {} disconnected", id );
                    break
                },
                 Ok( job ) => {
                    println!("worker {} got a job; executing", id );
                    job();
                }
            }
        });

        Worker {
            id,
            thread: Some( thread )
        }
    }
}


type Job = Box<dyn FnOnce() + Send + 'static>;


pub struct ThreadPool {
    sender : Option<mpsc::Sender<Job>>,
    workers: Vec<Worker>
}


impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new( s: usize) -> ThreadPool {
        assert!( s > 0 );

        let    ( sender, receiver ) = mpsc::channel();
        let      receiver           = Arc::new( Mutex::new( receiver ) );
        let mut  workers            = Vec::with_capacity( s );

        for id in 0..s {
            workers.push( Worker::new( id , Arc::clone( &receiver ) ) );
        }

        ThreadPool {
            sender: Some( sender ),
            workers
        }
    }

    pub fn execute<F>( &self, f: F )
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new( f );
        self.sender.as_ref().unwrap().send( job ).unwrap();
    }
}


impl Drop for ThreadPool {
    fn drop( &mut self ) {
        drop( self.sender.take() );

        for worker in &mut self.workers {
            println!( "Shutting down worker {}", worker.id );

            if let Some( thread ) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
