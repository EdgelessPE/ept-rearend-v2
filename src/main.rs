use actix_web::{get, http::header, http::StatusCode, web, App, HttpResponse, HttpServer, Result};
use regex::Regex;
use std::{fs, path::Path,os::windows::prelude::*};
use serde::{Deserialize, Serialize};

//常量配置
const DISK_DIRECTORY: &str = "E:\\Edgeless_Onedrive\\OneDrive - 洛阳科技职业学院";
const STATION_URL: &str = "https://pineapple.edgeless.top/disk";
const TOKEN: &str ="WDNMD";

//自定义Json结构
#[derive(Serialize, Deserialize)]
struct CateData {
    payload:Vec<String>
}
#[derive(Serialize, Deserialize)]
struct ListData {
    payload:Vec<ListObj>
}
#[derive(Serialize, Deserialize)]
struct ListObj {
    name:String,
    size:u64,
    node_type:String,
    url:String
}

//自定义请求参数结构体
#[derive(Deserialize)]
struct EptAddrQueryStruct {
    name: String,
    cate:String,
    version:String,
    author:String
}
#[derive(Deserialize)]
struct PluginListQueryStruct {
    name:String
}
#[derive(Deserialize)]
struct TokenRequiredQueryStruct{
    token:String
}

//工厂函数

#[get("/alpha/{quest}")]
async fn factory_alpha(web::Path(quest): web::Path<String>,info: web::Query<TokenRequiredQueryStruct>)->HttpResponse{
    //校验token
    if &info.token!=TOKEN{
        return return_error_query(String::from("Invalid token : ")+&info.token)
    }
    return match &quest[..] {
        "version" => return_text_result(get_alpha_version()),
        "addr" => return_redirect_result(get_alpha_addr()),
        _ => return_error_query(format!("/alpha/{}",quest))
    }
}

#[get("/info/{quest}")]
async fn factory_info(web::Path(quest): web::Path<String>) -> HttpResponse {
    return match &quest[..] {
        "iso_version" => return_text_result(get_iso_version()),
        "iso_addr" => return_redirect_result(get_iso_addr()),
        "hub_version" => return_text_result(get_hub_version()),
        "hub_addr" => return_redirect_result(get_hub_addr()),
        "ventoy_plugin_addr" => {
            return_redirect_string(String::from(STATION_URL) + "/Socket/Hub/ventoy_wimboot.img")
        }
        _ => return_error_query(quest)
    };
}

#[get("/plugin/cateData")]
async fn factory_plugin_cate() -> HttpResponse{
    return return_json_result(get_plugin_cate())
}

#[get("/plugin/listData")]
async fn factory_plugin_list(info: web::Query<PluginListQueryStruct>) -> HttpResponse {
    //判断目录是否存在
    if !Path::new(&(String::from(DISK_DIRECTORY)+"/插件包/"+&info.name.clone())).exists() {
        return return_error_query(String::from("No such cate"))
    }
    return return_json_result(get_plugin_list(info.name.clone()))
}

#[get("/ept/{quest}")]
async fn factory_ept(web::Path(quest): web::Path<String>) -> HttpResponse {
    return_text_string(format!("Ept, quest:{}", quest))
}

#[get("/misc/{quest}")]
async fn factory_misc(web::Path(quest): web::Path<String>) -> HttpResponse {
    return_text_string(format!("Misc, quest:{}", quest))
}

//主函数
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listen_addr = "127.0.0.1:8080";

    HttpServer::new(|| {
        App::new()
            .service(factory_info)
            .service(factory_alpha)
            .service(factory_plugin_cate)
            .service(factory_plugin_list)
            .service(factory_ept)
            .service(factory_misc)
    })
    .bind(listen_addr)?
    .run()
    .await
}

//文件选择器函数
fn file_selector(path: String, exp: String) -> Result<String, String> {
    //校验路径是否存在
    if !Path::new(&path).exists() {
        return Err(String::from("file_selector:Can't find ") + &path);
    }

    //校验正则表达式是否有效
    let expression = Regex::new(&exp);
    if let Err(_) = expression {
        return Err(String::from("file_selector:Invalid expression: ") + &exp);
    }

    //列出文件列表
    let file_list = fs::read_dir(&path);
    if let Err(_) = file_list {
        return Err(String::from("file_selector:Can't read as directory: ") + &path);
    }

    //遍历匹配文件名
    for entry in file_list.unwrap() {
        let file_name = entry.unwrap().file_name().clone();
        let true_name = file_name.to_str().unwrap();
        //println!("checking {}", &true_name);
        if regex::is_match(&exp, true_name).unwrap() {
            //println!("match {}", &true_name);
            return Ok(String::from(true_name));
        }
    }

    return Err(
        String::from("file_selector:Matched nothing when looking into ") + &path + " for " + &exp,
    );
}

//版本号提取器函数
fn version_extractor(name: String, index: usize) -> Result<String, String> {
    //首次切割，获取拓展名的值及其长度
    let mut ext_name = "";
    let mut ext_len = 0;
    let result_ext: Vec<&str> = name.split(".").collect();
    if result_ext.len() > 1 {
        ext_name = result_ext[result_ext.len() - 1];
        ext_len = ext_name.len();
    }

    //再次切割（去拓展名切割），获取字段，将拓展名叠加到最后
    let mut result: Vec<&str> = name[0..name.len() - ext_len - 1].split("_").collect();
    result.push(ext_name);

    if index > result.len() {
        return Err(
            String::from("version_extractor:Index out of range when split ")
                + &name
                + ",got "
                + &index.to_string(),
        );
    }
    //println!("{:?}",result);
    return Ok(result[index].to_string());
}

//按Text返回函数
fn return_text_result(content: Result<String, String>) -> HttpResponse {
    if let Err(error) = content {
        return return_error_internal(error);
    }
    return HttpResponse::Ok().body(format!("{}", content.unwrap()));
}
fn return_text_string(content: String) -> HttpResponse {
    return HttpResponse::Ok().body(content);
}

//按Redirect返回函数
fn return_redirect_result(url: Result<String, String>) -> HttpResponse {
    if let Err(error) = url {
        return return_error_internal(error);
    }
    return HttpResponse::Ok()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(header::LOCATION, url.unwrap())
        .finish();
}
fn return_redirect_string(url: String) -> HttpResponse {
    return HttpResponse::Ok()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(header::LOCATION, url)
        .finish();
}

//按Json返回函数
fn return_json_result<T: Serialize>(data:Result<T,String>) ->HttpResponse{
    if let Err(error) = data {
        return return_error_internal(error);
    }
    return HttpResponse::Ok()
        .json(data.unwrap())
}

//返回内部错误
fn return_error_internal(msg:String)->HttpResponse{
    return HttpResponse::Ok()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(format!("Error: Internal\n{}",msg))
}

//返回查询错误
fn return_error_query(msg:String)->HttpResponse{
    return HttpResponse::Ok()
        .status(StatusCode::BAD_REQUEST)
        .body(format!("Error: Quest\nUnknown quest:{}",msg))
}

//获取ISO版本号/info/iso_version
fn get_iso_version() -> Result<String, String> {
    //选中ISO文件
    let iso_name = file_selector(
        String::from(DISK_DIRECTORY) + "\\Socket",
        String::from("^Edgeless.*iso$"),
    )?;
    //提取版本号
    let iso_version = version_extractor(iso_name, 2)?;
    return Ok(iso_version);
}

//获取ISO下载地址/info/iso_addr
fn get_iso_addr() -> Result<String, String> {
    //选中ISO文件
    let iso_name = file_selector(
        String::from(DISK_DIRECTORY) + "\\Socket",
        String::from("^Edgeless.*iso$"),
    )?;
    //拼接并返回
    return Ok(STATION_URL.to_string() + "/Socket/" + &iso_name);
}

//获取Alpha版本wim文件版本号/info/alpha_version
fn get_alpha_version() -> Result<String, String> {
    //选中Alpha_xxx.wim文件
    let wim_name = file_selector(
        String::from(DISK_DIRECTORY) + "\\Socket\\Alpha",
        String::from("^Edgeless.*wim$"),
    )?;
    //提取版本号
    let wim_version = version_extractor(wim_name, 2)?;
    return Ok(wim_version);
}

//获取Alpha版本wim文件下载地址/info/alpha_addr
fn get_alpha_addr() -> Result<String, String> {
    //选中Alpha_xxx.wim文件
    let wim_name = file_selector(
        String::from(DISK_DIRECTORY) + "\\Socket\\Alpha",
        String::from("^Edgeless.*wim$"),
    )?;
    //拼接并返回
    return Ok(STATION_URL.to_string() + "/Socket/Alpha/" + &wim_name);
}

//获取Hub版本号/info/hub_version
fn get_hub_version() -> Result<String, String> {
    //选中Edgeless Hub_xxx.7z文件
    let hub_name = file_selector(
        String::from(DISK_DIRECTORY) + "\\Socket\\Hub",
        String::from("^Edgeless Hub.*7z$"),
    )?;
    //提取版本号
    let hub_version = version_extractor(hub_name, 2)?;
    return Ok(hub_version);
}

//获取Hub下载地址/info/hub_addr
fn get_hub_addr() -> Result<String, String> {
    //选中Edgeless Hub_xxx.7z文件
    let hub_name = file_selector(
        String::from(DISK_DIRECTORY) + "\\Socket\\Hub",
        String::from("^Edgeless Hub.*7z$"),
    )?;
    //拼接并返回
    return Ok(STATION_URL.to_string() + "/Socket/Hub/" + &hub_name);
}

//获取插件分类数组
fn get_plugin_cate() -> Result<CateData,String>{
    //扫描插件包目录
    let cate_list=fs::read_dir(DISK_DIRECTORY.to_string()+"/插件包");
    if let Err(_)=cate_list{
        return Err(String::from("get_plugin_cate:Fail to read : ")+&DISK_DIRECTORY+"/插件包");
    }

    //形成Vec<String>
    let mut result=Vec::new();
    for entry in cate_list.unwrap() {
        //解析node名称
        let file_name = entry.unwrap().file_name().clone();
        let true_name = file_name.to_str().unwrap();
        //判断是否为目录，是则push到Vector
        let path=String::from(DISK_DIRECTORY)+"/插件包/"+&true_name;
        if Path::new(&path).is_dir(){
            result.push(true_name.to_string());
        }
    }
    //println!("{:?}",result);
    return Ok(CateData {
        payload:result
    });
}

//获取分类详情
fn get_plugin_list(cate_name:String) -> Result<ListData,String>{
    //扫描分类目录
    let list=fs::read_dir(DISK_DIRECTORY.to_string()+"/插件包/"+&cate_name);
    if let Err(_)=list{
        return Err(String::from("get_plugin_list:Can't open as directory : ")+&DISK_DIRECTORY+"/插件包/"+&cate_name);
    }

    //形成Vec<ListObj>
    let mut result=Vec::new();
    for entry in list.unwrap(){
        //解析node名称
        let dir_entry=entry.unwrap();
        let file_name = &dir_entry.file_name().clone();
        let true_name = file_name.to_str().unwrap().to_string();

        //获取文件大小
        let meta_data = fs::metadata(&dir_entry.path());
        if let Err(_)=meta_data{
            return Err(String::from("get_plugin_list:Fail to read : ")+&DISK_DIRECTORY+"/插件包/"+&cate_name);
        }
        let file_size=meta_data.unwrap().file_size();

        //将后缀名为.7z的推入Vec
        if regex::is_match(".7z",&true_name).unwrap(){
            result.push(ListObj{
                name:true_name.clone(),
                size:file_size,
                node_type:String::from("FILE"),
                url:String::from(STATION_URL)+"/插件包/"+&true_name
            })
        }
    }
    return Ok(ListData {
        payload:result
    });
}