
#[derive(Debug)]
pub struct DbHolder{
    db:Db
}
#[derive(Clone,Debug)]
pub struct Db{

}

impl DbHolder{
    pub fn new()->Self{
        DbHolder{
            // TODO 待完善
            db:Db{}
        }
    }
    pub fn clone(&self)->Db{
        self.db.clone()
    }
}