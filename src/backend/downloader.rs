use crate::hal_resource::HalResource;
use halres_downloader::Resource;

pub(crate) fn make_resource(entry: Resource) -> HalResource {
    let Resource {
        url,
        title,
        description,
        timestamp,
    } = entry;
    let mut uri_entry = HalResource::default();
    uri_entry.title = title;
    uri_entry.auto_descr = description;
    uri_entry.live_status = String::from("0");
    uri_entry.url = url.as_str().into();
    uri_entry.uri_uuid = blake3::hash(uri_entry.url.as_bytes()).to_hex().to_string();
    uri_entry.scheme = url.scheme().into();
    uri_entry.host = url.host_str().unwrap_or("-").into();
    uri_entry.path = url.path().into();
    uri_entry.crea_time = timestamp.to_string();
    uri_entry.modi_time = timestamp.to_string();
    uri_entry
}

pub(crate) fn insert_resources(uris: &[HalResource]) {
    for (index, uri) in uris.iter().enumerate() {
        super::DB.with(|f| {
            f.prepare("INSERT INTO uris values (?,?,?,?,?,?,?,?,?,?,?,?,?);")
                .unwrap()
                .execute(rusqlite::params![
                    index.to_string(),
                    uri.url,
                    uri.scheme,
                    uri.host,
                    uri.path,
                    uri.live_status,
                    uri.title,
                    uri.auto_descr,
                    uri.man_descr,
                    uri.crea_user,
                    uri.crea_time,
                    uri.modi_user,
                    uri.modi_time
                ])
                .unwrap();
        });
    }
}
