use std::{sync::{mpsc, Arc, Mutex}, thread};


struct Worker {
    id    : usize,
    thread: thread::JoinHandle<()>,
}


impl Worker {
    fn new( id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>> ) -> Worker {
        let thread = thread::spawn( move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            job();
        });

        Worker { id, thread }
    }
}


type Job = Box<dyn FnOnce() + Send + 'static>;


pub struct ThreadPool {
    sender : mpsc::Sender<Job>,
    workers: Vec<Worker>,
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

        ThreadPool { sender, workers }
    }

    pub fn execute<F>( &self, f: F )
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new( f );
        self.sender.send( job ).unwrap();
    }
}
